mod views;

use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    response::Redirect,
    routing::{get, post},
    Form, Router,
};
use axum_htmx::HxLocation;
use chrono::{Duration, NaiveDate, Utc};
use futures::{stream::FuturesUnordered, StreamExt};
use maud::{html, Markup};
use serde::Deserialize;

use crate::{
    services::bet::{Bet, Payout},
    user::User,
    Ctx, MELBOURNE,
};

use self::views::bet_form::BetForm;

async fn index(State(ctx): State<Ctx>, user: User) -> Markup {
    let timezone = "Australia/Melbourne";

    // Work out what 'today' is in the local timezone
    let today = Utc::now()
        .with_timezone(&chrono_tz::Tz::from_str(&timezone).unwrap())
        .naive_local()
        .date();
    let next_week = today + Duration::weeks(1);

    let forecast = ctx
        .services
        .weather
        .get_forecast(today, next_week)
        .await
        .into_iter()
        .map(|(date, forecast)| {
            let bet = ctx.services.bet.find_bet(date);

            async move {
                (
                    date,
                    forecast,
                    bet.await.map(|bet| bet.wager).unwrap_or_default(),
                )
            }
        })
        .collect::<futures::stream::FuturesOrdered<_>>()
        .collect::<Vec<_>>()
        .await;

    let balance = user.data.balance;

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
    date: NaiveDate,
}

async fn get_bet_form(
    State(ctx): State<Ctx>,
    user: User,
    date: Option<Query<DateQueryParam>>,
) -> Markup {
    let date = date.map(|date| date.0.date);

    let (bet, payout, existing) = if let Some(date) = date {
        let forecast = &ctx
            .services
            .weather
            .get_daily_forecast(date, MELBOURNE)
            .await;

        fn round(n: f64, points: usize) -> f64 {
            let f = 10f64.powi(points as i32);
            (n * f).round() / f
        }

        let bet = ctx.services.bet.find_bet(date).await;

        let existing = bet.is_some();

        let bet = bet.unwrap_or_else(|| Bet {
            temperature: round(
                forecast.minimum_temperature
                    + ((forecast.maximum_temperature - forecast.minimum_temperature) / 2.0),
                2,
            ),
            range: 5.0,
            rain: forecast.rain > 0.5,
            wager: round(user.data.balance * 0.1, 2),
        });

        let payout = Payout::max_payout(&bet, date, forecast).total();
        (Some(bet.into()), payout, existing)
    } else {
        (None, 0.0, false)
    };

    views::bet_form::render(date, bet, payout, existing)
}

async fn place_bet(
    State(ctx): State<Ctx>,
    mut user: User,
    Path(date): Path<NaiveDate>,
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
        .get_daily_forecast(date, MELBOURNE)
        .await;
    let payout = Payout::max_payout(&bet, date, &forecast);

    ctx.services.bet.place(&mut user, date, bet, payout).await;

    Redirect::to("/")
}

async fn calculate_payout(
    State(ctx): State<Ctx>,
    Path(date): Path<NaiveDate>,
    Form(bet_form): Form<BetForm>,
) -> Markup {
    let forecast = ctx
        .services
        .weather
        .get_daily_forecast(date, MELBOURNE)
        .await;
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
                    let bet = ctx.services.bet.find_bet(*date);
                    let weather = ctx.services.weather.get_historical_weather(*date);

                    async move {
                        let weather = weather.await.unwrap();

                        views::payouts::Payout {
                            date: date.clone(),
                            bet: bet.await.unwrap(),
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
