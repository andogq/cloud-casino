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
use time::macros::datetime;
use tower_http::services::ServeDir;
use tower_sessions::{Expiry, MemoryStore, SessionManagerLayer};
use user::User;
use weather::{Point, WeatherService};

const MELBOURNE: Point = Point {
    latitude: -37.814,
    longitude: 144.9633,
};

async fn home(State(ctx): State<Ctx>, user: User) -> Markup {
    views::page(html! {
        (views::app::render(&user).await)

        (views::forecast::render(ctx.weather_service, MELBOURNE).await)
        (views::bet_form::render(&user).await)
    })
}

#[derive(Deserialize)]
struct CalculateInput {
    #[serde(default, deserialize_with = "CalculateInput::deserialize_rain")]
    rain: bool,
    temperature: f64,
    confidence: f64,
    wager: f64,
}

impl CalculateInput {
    pub fn deserialize_rain<'de, D>(d: D) -> Result<bool, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(String::deserialize(d)? == "on")
    }
}

async fn payout(Form(form): Form<CalculateInput>) -> Markup {
    const MAX_MULTIPLIER: f64 = 10.0;

    let payout_multiplier = MAX_MULTIPLIER * form.confidence / 100.0;

    let max_payout = payout_multiplier * form.wager;

    html! { (format!("{max_payout:.2}")) }
}

async fn forecast(State(ctx): State<Ctx>) -> Markup {
    views::forecast::render(ctx.weather_service, MELBOURNE).await
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
        .fallback_service(ServeDir::new("./static"))
        .layer(session_layer)
        .with_state(Ctx {
            db: pool,
            weather_service: WeatherService::new(),
        });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
