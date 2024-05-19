mod login;
mod views;

use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    response::Redirect,
    routing::{get, post},
    Form, Router,
};
use axum_htmx::{HxLocation, HxRetarget};
use chrono::{Duration, NaiveDate, Utc};
use chrono_tz::Australia::Melbourne;
use futures::{stream::FuturesUnordered, StreamExt};
use maud::{html, Markup};
use serde::Deserialize;

use crate::{
    app::views::{bet_form::BetFormVariant, login::Provider},
    services::bet::{Bet, Payout},
    user::UserId,
    Ctx, MELBOURNE,
};

use self::views::{bet_form::BetForm, forecast::ForecastDay};

async fn index(State(ctx): State<Ctx>, user_id: Option<UserId>) -> Markup {
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
            let bet = user_id.map(|user_id| ctx.services.bet.find_bet(user_id, date));

            async move {
                ForecastDay {
                    date,
                    forecast,
                    user_bet: if let Some(bet) = bet {
                        Some(bet.await.map(|bet| bet.wager).unwrap_or_default())
                    } else {
                        None
                    },
                }
            }
        })
        .collect::<futures::stream::FuturesOrdered<_>>()
        .collect::<Vec<_>>()
        .await;

    let (hero, ready_payouts) = if let Some(user_id) = user_id {
        let balance = {
            let balance = ctx.services.bet.get_balance(user_id).await;
            format!("${balance:.2}")
        };

        let ready_payouts = ctx.services.bet.get_ready(user_id).await.len();

        (balance, ready_payouts)
    } else {
        ("cloud casino".to_string(), 0)
    };

    views::page(views::shell::render(
        hero,
        ready_payouts,
        true,
        html! {
            (views::forecast::render(forecast, None, !user_id.is_some()))

            (views::home::render(
                (!user_id.is_some()).then_some(
                    &[Provider {
                        name: "GitHub".to_string(),
                        icon: "github".to_string(),
                        url: ctx
                            .services
                            .oauth
                            .generate_authorization_url("github")
                            .await
                            .unwrap()
                            .to_string(),
                    }]
                ),
            ))
        },
    ))
}

#[derive(Deserialize)]
pub struct DateQueryParam {
    date: NaiveDate,
}

async fn get_bet_form(
    State(ctx): State<Ctx>,
    user_id: UserId,
    date: Option<Query<DateQueryParam>>,
) -> Markup {
    let Some(date) = date.map(|date| date.0.date) else {
        return views::home::render(None);
    };

    let balance = ctx.services.bet.get_balance(user_id).await;

    let forecast = &ctx
        .services
        .weather
        .get_daily_forecast(date, MELBOURNE)
        .await;

    fn round(n: f64, points: usize) -> f64 {
        let f = 10f64.powi(points as i32);
        (n * f).round() / f
    }

    let bet = ctx.services.bet.find_bet(user_id, date).await;

    let existing = bet.is_some();
    let today = Utc::now().with_timezone(&Melbourne).naive_local().date();
    let bet_form_variant = if date == today {
        BetFormVariant::Today
    } else if existing {
        BetFormVariant::Replace
    } else {
        BetFormVariant::Normal
    };

    let bet = bet.unwrap_or_else(|| Bet {
        temperature: round(
            forecast.minimum_temperature
                + ((forecast.maximum_temperature - forecast.minimum_temperature) / 2.0),
            2,
        ),
        range: 5.0,
        rain: forecast.rain > 0.5,
        wager: if let BetFormVariant::Today = bet_form_variant {
            // Today is selected, but no bet provided
            0.0
        } else {
            // A future day is provided, pre-fill a wager
            round(balance * 0.1, 2)
        },
    });

    let payout = Payout::max_payout(&bet, date, forecast).total();

    views::bet_form::render(Some(date), Some(bet.into()), payout, bet_form_variant)
}

async fn place_bet(
    State(ctx): State<Ctx>,
    user_id: UserId,
    Path(date): Path<NaiveDate>,
    Form(bet_form): Form<BetForm>,
) -> Result<Redirect, (HxRetarget, String)> {
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

    ctx.services
        .bet
        .place(user_id, date, bet, payout)
        .await
        .map_err(|bet_error| {
            (
                HxRetarget("#maximum-payout".to_string()),
                bet_error.to_string(),
            )
        })?;

    Ok(Redirect::to("/"))
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

async fn payout(State(ctx): State<Ctx>, user_id: UserId) -> Markup {
    let balance = ctx.services.bet.get_balance(user_id).await;

    let ready_payouts = ctx.services.bet.get_ready(user_id).await;

    views::page(views::shell::render(
        format!("${balance:.2}"),
        ready_payouts.len(),
        false,
        views::payouts::render(
            &ready_payouts
                .iter()
                .map(|(date, outcome)| {
                    let bet = ctx.services.bet.find_bet(user_id, *date);
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

async fn perform_payout(State(ctx): State<Ctx>, user_id: UserId) -> (HxLocation, &'static str) {
    ctx.services.bet.payout(user_id).await;

    (HxLocation::from_str("/").unwrap(), "redirecting")
}

pub fn init() -> Router<Ctx> {
    Router::new()
        .route("/", get(index))
        .route("/bet", get(get_bet_form))
        .route("/bet/:date", post(place_bet))
        .route("/bet/:date/payout", get(calculate_payout))
        .route("/payout", get(payout).post(perform_payout))
        .nest("/login", login::init())
}
