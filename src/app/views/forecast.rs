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

#[derive(Clone, Debug)]
pub struct ForecastDay {
    pub date: NaiveDate,
    pub forecast: Forecast,
    pub user_bet: Option<f64>,
}

pub fn render(days: Vec<ForecastDay>, selected: Option<NaiveDate>, disabled: bool) -> Markup {
    html! {
        form #forecast
            hx-get="/bet" hx-trigger="change"
            hx-target=".bet-form-target" hx-indicator=".bet-form-target"
            hx-swap="outerHTML" {
            .days {
                label .deselect {
                    input type="radio" name="date" value="null" checked[selected.is_none()];
                }

                @for ForecastDay { date, forecast, user_bet } in days {
                    @let checked = selected.map(|d| d == date).unwrap_or(false);
                    label .weather-tile {
                        input type="radio" name="date" autocomplete="off"
                            value=(date) checked[checked] disabled[disabled];

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


                        @if let Some(bet_placed) = user_bet {
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
}
