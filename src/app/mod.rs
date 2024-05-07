mod views;

use axum::{
    extract::{Path, Query, State},
    response::Redirect,
    routing::{get, post},
    Form, Router,
};
use axum_htmx::HxLocation;
use futures::{stream::FuturesUnordered, StreamExt};
use maud::{html, Markup};
use serde::Deserialize;
use time::{Date, OffsetDateTime};

use crate::{
    services::bet::{Bet, Payout},
    user::User,
    Ctx, MELBOURNE,
};

use self::views::bet_form::BetForm;

async fn index(State(ctx): State<Ctx>, user: User) -> Markup {
    let balance = user.data.balance;
    let forecast = ctx
        .services
        .weather
        .get_forecast(MELBOURNE)
        .await
        .into_iter()
        .map(|forecast| {
            let placed = user
                .data
                .bets
                .get(&forecast.date)
                .map(|bet| bet.bet.wager)
                .unwrap_or_default();
            (forecast, placed)
        })
        .collect();

    // TODO: Get rid of this
    dbg!(
        ctx.services
            .new_weather
            .get_forecast(OffsetDateTime::now_utc())
            .await
    );

    let ready_payouts = ctx.services.bet.get_ready(&user).await.len();

    let payout = 0.0;

    views::page(views::shell::render(
        balance,
        ready_payouts,
        true,
        html! {
            (views::forecast::render(forecast, None))

            (views::bet_form::render(None, None, payout, false))
        },
    ))
}

#[derive(Deserialize)]
pub struct DateQueryParam {
    date: Date,
}

async fn get_bet_form(
    State(ctx): State<Ctx>,
    user: User,
    date: Option<Query<DateQueryParam>>,
) -> Markup {
    let date = date.map(|date| date.0.date);

    let (bet, payout, existing) = if let Some(date) = date {
        let day_i = (date - OffsetDateTime::now_utc().date()).whole_days() as usize;
        let forecast = &ctx.services.weather.get_forecast(MELBOURNE).await[day_i];

        fn round(n: f64, points: usize) -> f64 {
            let f = 10f64.powi(points as i32);
            (n * f).round() / f
        }

        let bet = user.data.bets.get(&date).map(|record| record.bet.clone());

        let existing = bet.is_some();

        let bet = bet.unwrap_or_else(|| Bet {
            temperature: round(forecast.min + ((forecast.max - forecast.min) / 2.0), 2),
            range: 5.0,
            rain: forecast.rain > 0.5,
            wager: round(user.data.balance * 0.1, 2),
        });

        let payout = Payout::max_payout(&bet, date, &forecast).total();
        (Some(bet.into()), payout, existing)
    } else {
        (None, 0.0, false)
    };

    views::bet_form::render(date, bet, payout, existing)
}

async fn place_bet(
    State(ctx): State<Ctx>,
    mut user: User,
    Path(date): Path<Date>,
    Form(bet_form): Form<BetForm>,
) -> Redirect {
    // Construct the bet
    let bet = Bet {
        temperature: bet_form.temperature,
        range: bet_form.range,
        rain: bet_form.rain,
        wager: bet_form.wager,
    };

    // Determine the forecast for the day
    let forecast = ctx
        .services
        .weather
        .get_forecast(MELBOURNE)
        .await
        .into_iter()
        .find(|forecast| forecast.date == date)
        .unwrap();
    let payout = Payout::max_payout(&bet, date, &forecast);

    ctx.services.bet.place(&mut user, date, bet, payout).await;

    Redirect::to("/")
}

async fn calculate_payout(
    State(ctx): State<Ctx>,
    Path(date): Path<Date>,
    Form(bet_form): Form<BetForm>,
) -> Markup {
    let forecast = ctx
        .services
        .weather
        .get_forecast(MELBOURNE)
        .await
        .into_iter()
        .find(|forecast| forecast.date == date)
        .unwrap();
    let payout = Payout::max_payout(&bet_form.into(), date, &forecast);

    views::bet_form::render_maximum_payout(date, payout.total())
}

async fn payout(State(ctx): State<Ctx>, user: User) -> Markup {
    let balance = user.data.balance;

    let ready_payouts = ctx.services.bet.get_ready(&user).await;

    views::page(views::shell::render(
        balance,
        ready_payouts.len(),
        false,
        views::payouts::render(
            &ready_payouts
                .iter()
                .map(|(date, outcome)| {
                    let bet_record = user.data.bets.get(date).unwrap().clone();
                    let weather = ctx.services.weather.get_historical(MELBOURNE, date.clone());

                    async move {
                        let weather = weather.await.unwrap();

                        views::payouts::Payout {
                            date: date.clone(),
                            bet: bet_record.bet,
                            rain: weather.rain,
                            rain_correct: outcome.rain,
                            temperature: weather.temperature,
                            temperature_correct: outcome.temperature,
                            payout: outcome.payout,
                        }
                    }
                })
                .collect::<FuturesUnordered<_>>()
                .collect::<Vec<_>>()
                .await,
        ),
    ))
}

async fn perform_payout(State(ctx): State<Ctx>, mut user: User) -> (HxLocation, &'static str) {
    ctx.services.bet.payout(&mut user).await;

    (HxLocation::from_str("/").unwrap(), "redirecting")
}

pub fn init() -> Router<Ctx> {
    Router::new()
        .route("/", get(index))
        .route("/bet", get(get_bet_form))
        .route("/bet/:date", post(place_bet))
        .route("/bet/:date/payout", get(calculate_payout))
        .route("/payout", get(payout).post(perform_payout))
}
