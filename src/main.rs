mod db;
mod user;
mod views;
mod weather;

use axum::{
    extract::State,
    routing::{get, post},
    Form, Router,
};
use maud::{html, Markup};
use serde::{Deserialize, Deserializer};
use sqlx::SqlitePool;
use time::{macros::datetime, Date, OffsetDateTime};
use tower_http::services::ServeDir;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use user::User;
use weather::{Point, WeatherService};

use crate::user::{Bet, Payout};

const MELBOURNE: Point = Point {
    latitude: -37.814,
    longitude: 144.9633,
};

async fn home(State(ctx): State<Ctx>, user: User) -> Markup {
    views::page(html! {
        (views::app::render(&user).await)

        form #bet-form hx-boost="true" action="/bet" method="post" {
            .payout-preview hx-post="/payout" hx-target="#payout" hx-trigger="input" {
                (views::forecast::render(&user, ctx.weather_service, MELBOURNE).await)

                .controls .card {
                    (views::bet_form::render(&user).await)
                }
            }
        }
    })
}

#[derive(Deserialize)]
struct BetForm {
    #[serde(default, deserialize_with = "BetForm::deserialize_rain")]
    rain: bool,
    min: f64,
    max: f64,
    wager: f64,
    date: Date,
}

impl BetForm {
    pub fn deserialize_rain<'de, D>(d: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(String::deserialize(d)? == "on")
    }
}

const MAX_TEMPERATURE_MULTIPLIER: f64 = 5.0;
const DAY_MULTIPLIER: f64 = 0.2;
const RAIN_MULTIPLIER: f64 = 1.25;

fn temperature_multiplier(forecast_range: f64, guess_range: f64) -> f64 {
    MAX_TEMPERATURE_MULTIPLIER / (guess_range / forecast_range)
}

fn rain_multiplier(correct: bool) -> f64 {
    if correct {
        RAIN_MULTIPLIER
    } else {
        0.0
    }
}

fn day_multiplier(days_ahead: i64) -> f64 {
    DAY_MULTIPLIER * days_ahead as f64
}

fn calculate_max_payout(wager: f64, date: Date, forecast: (f64, f64), guess: (f64, f64)) -> f64 {
    let multiplier = temperature_multiplier(forecast.1 - forecast.0, guess.1 - guess.0)
        + rain_multiplier(true)
        + day_multiplier((date - OffsetDateTime::now_utc().date()).whole_days());

    multiplier * wager
}

async fn payout(State(ctx): State<Ctx>, Form(form): Form<BetForm>) -> Markup {
    let forecast = ctx
        .weather_service
        .get_forecast(MELBOURNE)
        .await
        .into_iter()
        .find(|forecast| forecast.date == form.date)
        .unwrap();

    let max_payout = calculate_max_payout(
        form.wager,
        form.date,
        (forecast.min, forecast.max),
        (form.min, form.max),
    );

    html! { (format!("{max_payout:.2}")) }
}

async fn perform_payout(State(ctx): State<Ctx>, mut user: User) -> Markup {
    // Get all outstanding bets that have passed
    let now = OffsetDateTime::now_utc().date();

    let mut total_payout = 0.0;

    let payout_dates = std::mem::take(&mut user.data.outstanding_bets)
        .into_iter()
        .filter(|date| date < &&now);

    for date in payout_dates.clone() {
        let bet = user.data.bets.get_mut(&date).unwrap();
        let historical = ctx
            .weather_service
            .get_historical(MELBOURNE, date)
            .await
            .unwrap();

        let mut multiplier = rain_multiplier(bet.rain == historical.rain)
            + day_multiplier((date - bet.placed.date()).whole_days());

        if bet.min <= historical.temperature && bet.max >= historical.temperature {
            multiplier += temperature_multiplier(bet.forecast_range, bet.max - bet.min);
        }

        let payout_amount = bet.wager * multiplier;

        bet.payout = Some(Payout {
            date: OffsetDateTime::now_utc(),
            amount: payout_amount,
        });

        total_payout += payout_amount;
    }

    user.data.balance += total_payout;
    user.update_session().await;

    views::page(html! {
        .card {
            h1 { "Payout" }

            h2 { "Winnings: " (format!("${total_payout:.2}")) }

            ul {
                @for date in payout_dates {
                    li {
                        (date.to_string()) ": " (format!("${:.2}", user.data.bets.get(&date).unwrap().payout.as_ref().unwrap().amount))
                    }
                }
            }
        }
    })
}

async fn place_bet(State(ctx): State<Ctx>, mut user: User, Form(form): Form<BetForm>) -> Markup {
    let forecast = ctx
        .weather_service
        .get_forecast(MELBOURNE)
        .await
        .into_iter()
        .find(|forecast| forecast.date == form.date)
        .unwrap();

    // Place a bet on the user
    let bet = Bet {
        wager: form.wager,
        rain: form.rain,
        min: form.min,
        max: form.max,
        forecast_range: forecast.max - forecast.min,
        placed: OffsetDateTime::now_utc(),
        payout: None,
    };

    let previous_bet = user.data.bets.insert(form.date, bet);

    if let Some(previous_bet) = previous_bet {
        // Restore user's previous balance
        user.data.balance += previous_bet.wager;
    } else {
        // We need to indicate that this is an outstanding bet
        user.data.outstanding_bets.push(form.date);
    }

    // Take the money from the user
    user.data.balance -= form.wager;

    user.update_session().await;

    views::page(html! {
        (views::app::render(&user).await)

        .card {
            h1 { "Bet placed" }
        }
    })
}

async fn forecast(State(ctx): State<Ctx>, user: User) -> Markup {
    views::forecast::render(&user, ctx.weather_service, MELBOURNE).await
}

async fn summary(user: User) -> Markup {
    views::app::render(&user).await
}

#[derive(Clone)]
pub struct Ctx {
    pub db: SqlitePool,
    pub weather_service: WeatherService,
}

#[tokio::main]
async fn main() {
    // DB
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    // db::initialise(&pool).await;

    // Set up sessions
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::AtDateTime(datetime!(2099 - 01 - 01 0:00 UTC)));

    let app = Router::new()
        .route("/", get(home))
        .route("/payout", get(perform_payout).post(payout))
        .route("/forecast", get(forecast))
        .route("/bet", post(place_bet))
        .route("/summary", get(summary))
        .fallback_service(ServeDir::new("./static"))
        .layer(session_layer)
        .with_state(Ctx {
            db: pool,
            weather_service: WeatherService::new(),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
