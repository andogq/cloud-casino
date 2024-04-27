mod db;
mod weather;

use axum::{
    async_trait,
    extract::{FromRequestParts, State},
    http::request::Parts,
    routing::{get, post},
    Form, Router,
};
use maud::{html, Markup};
use reqwest::StatusCode;
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::SqlitePool;
use time::{macros::datetime, OffsetDateTime};
use tower_http::services::ServeDir;
use tower_sessions::{Expiry, MemoryStore, Session, SessionManagerLayer};
use weather::{Point, WeatherService};

use crate::weather::render_forecast;

const MELBOURNE: Point = Point {
    latitude: -37.814,
    longitude: 144.9633,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    last_request: OffsetDateTime,
}

impl User {
    const SESSION_KEY: &'static str = "user.data";

    pub async fn from_session(session: &Session) -> Option<Self> {
        session.get::<Self>(Self::SESSION_KEY).await.unwrap()
    }

    pub async fn update_session(&self, session: &Session) {
        session
            .insert(Self::SESSION_KEY, self.clone())
            .await
            .unwrap();
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            last_request: OffsetDateTime::now_utc(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    /// Perform the extraction.
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;

        let mut user = Self::from_session(&session).await.unwrap_or_default();
        user.last_request = OffsetDateTime::now_utc();
        user.update_session(&session).await;

        Ok(user)
    }
}

fn page(body: Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                link rel="stylesheet" type="text/css" href="/main.css";

                script defer src="//unpkg.com/alpinejs" {}
                script defer src="//unpkg.com/htmx.org" {}
            }

            body { (body) }
        }
    }
}

async fn home(State(ctx): State<Ctx>, user: User) -> Markup {
    page(html! {
        (render_forecast(ctx.weather_service, MELBOURNE).await)

        form #bet-form .card hx-post="/payout" hx-target="#payout" hx-trigger="input delay:0.5s" {
            h1 style="grid-area: title" { "Place Your Bets" }

            label style="grid-area: temperature" {
                "Temperature: "
                br;
                input type="number" name="temperature" value="20";
            }

            label style="grid-area: rain" {
                "Will it rain?"
                input type="checkbox" name="rain";
            }

            label style="grid-area: confidence" x-data="{ value: '0' }" {
                "Confidence: "
                span x-text="value" {}
                "%"

                br;

                input type="range" name="confidence" min="0" max="100" step="5" value="50" x-model="value";
            }

            label style="grid-area: wager" {
                "Wager: "
                br;
                input type="number" name="wager" min="0" value="10";
            }

            p style="grid-area: payout; text-align: right" {
                "Maximum potential payout: $"
                span #payout { "0.00" }
            }

            button style="grid-area: submit" type="submit" { "Bet" }
        }

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
    render_forecast(ctx.weather_service, MELBOURNE).await
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
