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

use crate::user::Bet;

const MELBOURNE: Point = Point {
    latitude: -37.814,
    longitude: 144.9633,
};

async fn home(State(ctx): State<Ctx>, user: User) -> Markup {
    views::page(html! {
        (views::app::render(&user).await)

        form #bet-form hx-boost="true" action="/bet" method="post" {
            .payout-preview hx-post="/payout" hx-target="#payout" hx-trigger="input" {
                (views::forecast::render(ctx.weather_service, MELBOURNE).await)

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
    temperature: f64,
    confidence: f64,
    wager: f64,
    day: Date,
}

impl BetForm {
    pub fn deserialize_rain<'de, D>(d: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(String::deserialize(d)? == "on")
    }
}

fn calculate_payout(wager: f64, date: Date, confidence: f64) -> f64 {
    const MAX_MULTIPLIER: f64 = 5.0;
    const DAY_MULTIPLIER: f64 = 0.2;
    const RAIN_MULTIPLIER: f64 = 1.25;

    let days_ahead = (date - OffsetDateTime::now_utc().date()).whole_days();
    let payout_multiplier = (MAX_MULTIPLIER * confidence / 100.0)
        + (DAY_MULTIPLIER * days_ahead as f64)
        + RAIN_MULTIPLIER;

    payout_multiplier * wager
}

async fn payout(Form(form): Form<BetForm>) -> Markup {
    let max_payout = calculate_payout(form.wager, form.day, form.confidence);

    html! { (format!("{max_payout:.2}")) }
}

async fn place_bet(mut user: User, Form(form): Form<BetForm>) -> Markup {
    // Place a bet on the user
    let bet = Bet {
        wager: form.wager,
        rain: form.rain,
        temperature: form.temperature,
        confidence: form.confidence,
        placed: OffsetDateTime::now_utc(),
        pay_out: None,
    };

    let previous_bet = user.data.bets.insert(form.day, bet);

    if let Some(previous_bet) = previous_bet {
        // Restore user's previous balance
        user.data.balance += previous_bet.wager;
    } else {
        // We need to indicate that this is an outstanding bet
        user.data.outstanding_bets.push(form.day);
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

async fn forecast(State(ctx): State<Ctx>) -> Markup {
    views::forecast::render(ctx.weather_service, MELBOURNE).await
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
        .route("/payout", post(payout))
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
