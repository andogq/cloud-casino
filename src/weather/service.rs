use std::time::Duration;

use moka::future::Cache;
use reqwest::Url;
use serde::Deserialize;
use time::Date;

use super::{DayWeather, Forecast, Point};

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
            wmo_code,
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
                                "weather_code",
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
            .json::<Response<DailyForecast>>()
            .await
            .unwrap()
            .daily;

        let forecast = date
            .into_iter()
            .zip(max)
            .zip(min)
            .zip(rain.into_iter().map(|rain| rain / 100.0))
            .zip(wmo_code)
            .map(|((((date, max), min), rain), wmo_code)| Forecast {
                date,
                rain,
                min,
                max,
                wmo_code,
            })
            .collect::<Vec<_>>();

        self.cache.insert(fixed, forecast.clone()).await;

        forecast
    }

    pub async fn get_historical(&self, location: Point, date: Date) -> Option<DayWeather> {
        let fixed = location.to_fixed();

        // TODO: Use cache

        let params = [
            ("latitude", location.latitude.to_string()),
            ("longitude", location.longitude.to_string()),
            ("start_date", date.to_string()),
            ("end_date", date.to_string()),
            (
                "daily",
                ["temperature_2m_mean", "precipitation_sum"].join(","),
            ),
            ("timezone", "auto".to_string()),
        ];

        // First attempt historical API
        let historical = self
            .client
            .get({
                Url::parse_with_params("https://archive-api.open-meteo.com/v1/archive", &params)
                    .unwrap()
                    .to_string()
            })
            .send()
            .await
            .unwrap()
            .json::<Response<Historical>>()
            .await
            .unwrap()
            .daily;

        let (temperature, rain) = if let (Some(temperature), Some(rain)) =
            (historical.temperature[0], historical.rain[0])
        {
            (temperature, rain)
        } else {
            // Try again with forecast API
            let forecast = self
                .client
                .get({
                    Url::parse_with_params("https://api.open-meteo.com/v1/forecast", &params)
                        .unwrap()
                        .to_string()
                })
                .send()
                .await
                .unwrap()
                .json::<Response<Historical>>()
                .await
                .unwrap()
                .daily;

            (forecast.temperature[0]?, forecast.rain[0]?)
        };

        Some(DayWeather {
            temperature,

            // Little buffer if there's a tiny amount of rain
            rain: rain > 0.01,
        })
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

    #[serde(rename = "weather_code")]
    wmo_code: Vec<usize>,
}

#[derive(Deserialize)]
struct Historical {
    #[serde(rename = "temperature_2m_mean")]
    temperature: Vec<Option<f64>>,

    #[serde(rename = "precipitation_sum")]
    rain: Vec<Option<f64>>,
}

#[derive(Deserialize)]
struct Response<T> {
    daily: T,
}
