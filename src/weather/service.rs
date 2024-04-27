use std::time::Duration;

use moka::future::Cache;
use reqwest::Url;
use serde::Deserialize;
use time::Date;

use super::{Forecast, Point};

#[derive(Clone)]
pub struct WeatherService {
    client: reqwest::Client,
    cache: Cache<(String, String), Vec<Forecast>>,
}

impl WeatherService {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: Cache::builder()
                .time_to_live(Duration::from_secs(60 * 60))
                .max_capacity(100)
                .build(),
        }
    }

    pub async fn get_forecast(&self, location: Point) -> Vec<Forecast> {
        let fixed = location.to_fixed();

        if let Some(forecast) = self.cache.get(&fixed).await {
            return forecast;
        }

        let DailyForecast {
            date,
            max,
            min,
            rain,
        } = self
            .client
            .get({
                Url::parse_with_params(
                    "https://api.open-meteo.com/v1/forecast",
                    &[
                        ("latitude", location.latitude.to_string()),
                        ("longitude", location.longitude.to_string()),
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
            .send()
            .await
            .unwrap()
            .json::<Response>()
            .await
            .unwrap()
            .daily;

        let forecast = date
            .into_iter()
            .zip(max)
            .zip(min)
            .zip(rain.into_iter().map(|rain| rain / 100.0))
            .map(|(((date, max), min), rain)| Forecast {
                date,
                rain,
                min,
                max,
            })
            .collect::<Vec<_>>();

        self.cache.insert(fixed, forecast.clone()).await;

        forecast
    }
}

#[derive(Deserialize)]
struct DailyForecast {
    #[serde(rename = "time")]
    date: Vec<Date>,

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
