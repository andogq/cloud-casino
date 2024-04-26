mod weather;

use axum::{
    routing::{get, post},
    Form, Router,
};
use maud::{html, Markup};
use serde::{Deserialize, Deserializer};
use tower_http::services::ServeDir;
use weather::Forecast;

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

async fn home() -> Markup {
    let forecast = Forecast::get(-37.814, 144.9633).await;

    page(html! {
        h1 { "This Week's Forecast" }

        #forecast {
            @for day in &forecast {
                .day {
                    h2 { (day.date.to_string())}

                    ul {
                        li {
                            "Min Temperature: "
                            b { (day.min) }
                        }

                        li {
                            "Max Temperature: "
                            b { (day.max) }
                        }

                        li {
                            "Chance of Rain:"
                            b {
                                ((day.rain * 100.0).round() as u32)
                                "%"
                            }
                        }
                    }
                }
            }
        }

        h1 { "Bet" }

        form hx-post="/" hx-target="#result" hx-trigger="input delay:0.5s" {
            label {
                "Date: "
                select name="day" {
                    @for day in forecast {
                        option { (day.date.to_string()) }
                    }
                }
            }

            br;

            label {
                input type="checkbox" name="rain";
                "Will it rain?"
            }

            br;

            label {
                "Temperature Guess: "
                input type="number" name="temperature" value="20";
            }

            br;

            label x-data="{ value: '0' }" {
                "Temperature Confidence: "
                span x-text="value" {}
                "%"

                br;

                input type="range" name="confidence" min="0" max="100" step="5" value="50" x-model="value";
            }

            br;

            label {
                "Wager: "
                input type="number" name="wager" min="0" value="10";
            }

            br;

            button type="submit" { "Bet" }
        }

        #result {}
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

async fn calculate(Form(form): Form<CalculateInput>) -> Markup {
    const MAX_MULTIPLIER: f64 = 10.0;

    let payout_multiplier = MAX_MULTIPLIER * form.confidence / 100.0;

    let max_payout = payout_multiplier * form.wager;

    html! {
        p {
            "Max payout: $"
            (format!("{max_payout:.2}"))
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(home))
        .route("/", post(calculate))
        .fallback_service(ServeDir::new("./static"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
