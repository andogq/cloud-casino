mod forecast;

use chrono::NaiveDate;
use reqwest::Url;
use serde::Deserialize;

pub use forecast::render_forecast;

#[derive(Debug)]
pub struct Forecast {
    pub date: NaiveDate,
    pub rain: f64,
    pub min: f64,
    pub max: f64,
}

impl Forecast {
    pub async fn get(latitude: f64, longitude: f64) -> Vec<Self> {
        #[derive(Deserialize)]
        struct DailyForecast {
            #[serde(rename = "time")]
            date: Vec<NaiveDate>,

            #[serde(rename = "temperature_2m_max")]
            max: Vec<f64>,

            #[serde(rename = "temperature_2m_min")]
            min: Vec<f64>,

            #[serde(rename = "precipitation_probability_mean")]
            rain: Vec<f64>,
        }

        #[derive(Deserialize)]
        struct Response {
            daily: DailyForecast,
        }

        let DailyForecast {
            date,
            max,
            min,
            rain,
        } = reqwest::get({
            Url::parse_with_params(
                "https://api.open-meteo.com/v1/forecast",
                &[
                    ("latitude", latitude.to_string()),
                    ("longitude", longitude.to_string()),
                    (
                        "daily",
                        [
                            "temperature_2m_max",
                            "temperature_2m_min",
                            "precipitation_probability_mean",
                        ]
                        .join(","),
                    ),
                    ("timezone", "auto".to_string()),
                ],
            )
            .unwrap()
            .to_string()
        })
        .await
        .unwrap()
        .json::<Response>()
        .await
        .unwrap()
        .daily;

        date.into_iter()
            .zip(max)
            .zip(min)
            .zip(rain.into_iter().map(|rain| rain / 100.0))
            .map(|(((date, max), min), rain)| Forecast {
                date,
                rain,
                min,
                max,
            })
            .collect()
    }
}
