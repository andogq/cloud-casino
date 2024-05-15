use chrono::NaiveDate;
use maud::{html, Markup};

use crate::services::weather::{Forecast, WeatherCode};

impl WeatherCode {
    pub fn to_lucide_icon(&self) -> &'static str {
        match self {
            WeatherCode::Sun => "sun",
            WeatherCode::PartialSun => "cloud-sun",
            WeatherCode::Cloud => "cloud",
            WeatherCode::Fog => "cloud-fog",
            WeatherCode::Drizzle => "cloud-drizzle",
            WeatherCode::Rain => "cloud-rain",
            WeatherCode::Snow => "snowflake",
            WeatherCode::Lightning => "cloud-lightning",
            WeatherCode::Unknown => "circle-help",
        }
    }
}

pub fn render(days: Vec<(NaiveDate, Forecast, f64)>, selected: Option<NaiveDate>) -> Markup {
    html! {
        form #forecast
            hx-get="/bet" hx-trigger="change"
            hx-target="#bet-form" hx-swap="outerHTML"
            hx-indicator="#bet-form" {
            .days {
                label .deselect {
                    input type="radio" name="date" value="null" checked[selected.is_none()];
                }

                @for (date, forecast, bet_placed) in days {
                    @let checked = selected.map(|d| d == date).unwrap_or(false);
                    label .weather-tile {
                        input type="radio" name="date" autocomplete="off"
                            value=(date) checked[checked];

                        p .day {
                            (date.format("%a").to_string())
                        }

                        i data-lucide=(forecast.weather_code.to_lucide_icon()) {}

                        .line .rain {
                            p { (format!("{:.0}%", forecast.rain * 100.0)) }
                            i data-lucide="droplets" {}
                        }

                        .line .temperature {
                            p { (format!("{:.0}° / {:.0}°", forecast.minimum_temperature, forecast.maximum_temperature)) }
                            i data-lucide="thermometer" {}
                        }

                        .line .bet-amount {
                            p { (format!("${bet_placed:.2}"))}
                            i data-lucide="badge-dollar-sign" {}
                        }
                    }
                }
            }
        }
    }
}
