mod weather;

use axum::{
    extract::State,
    routing::{get, post},
    Form, Router,
};
use maud::{html, Markup};
use serde::{Deserialize, Deserializer};
use tower_http::services::ServeDir;
use weather::{Point, Service};

use crate::weather::render_forecast;

const MELBOURNE: Point = Point {
    latitude: -37.814,
    longitude: 144.9633,
};

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

async fn home(State(service): State<Service>) -> Markup {
    page(html! {
        (render_forecast(service, MELBOURNE).await)

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

async fn forecast(State(service): State<Service>) -> Markup {
    render_forecast(service, MELBOURNE).await
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home))
        .route("/payout", post(payout))
        .route("/forecast", get(forecast))
        .fallback_service(ServeDir::new("./static"))
        .with_state(Service::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
