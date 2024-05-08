use std::fmt::Display;

use chrono::NaiveDate;
use reqwest::{Client, Url};
use serde::Deserialize;
use serde_json::Value;

use super::Forecast;

#[derive(Clone)]
pub struct Api {
    client: Client,
}

impl Api {
    /// The threshold in which it is considered that it will rain. This is intended to give a small
    /// buffer incase the rain isn't perceptable.
    const RAIN_THRESHOLD: f64 = 0.05;

    pub fn new(client: Client) -> Self {
        Self { client }
    }

    /// Get the forecast for a given date and location.
    pub async fn get_daily_forecast(&self, date: NaiveDate, location: (f64, f64)) -> Forecast {
        self.get_forecast(date, date, location).await.remove(0).1
    }

    /// Get the forecast for a given range of dates in a location.
    pub async fn get_forecast(
        &self,
        start: NaiveDate,
        end: NaiveDate,
        location: (f64, f64),
    ) -> Vec<(NaiveDate, Forecast)> {
        #[derive(Deserialize)]
        struct ForecastResponse {
            temperature_2m_min: Vec<f64>,
            temperature_2m_max: Vec<f64>,
            precipitation_probability_mean: Vec<f64>,
            weather_code: Vec<i64>,
            date: Vec<NaiveDate>,
        }

        let response = self
            .request::<ForecastResponse>(
                ApiSource::Forecast,
                Request {
                    start_date: start,
                    end_date: end,
                    latitude: location.0,
                    longitude: location.1,
                    parameters: &[
                        "weather_code",
                        "temperature_2m_min",
                        "temperature_2m_max",
                        "precipitation_probability_mean",
                    ],
                },
            )
            .await;

        (0..)
            .map_while(|i| {
                Some((
                    *response.date.get(i)?,
                    Forecast {
                        rain: (response.precipitation_probability_mean.get(i)? / 100.0)
                            > Self::RAIN_THRESHOLD,
                        minimum_temperature: *response.temperature_2m_min.get(i)?,
                        maximum_temperature: *response.temperature_2m_max.get(i)?,
                        weather_code: (*response.weather_code.get(i)?).into(),
                    },
                ))
            })
            .collect()
    }

    /// Internal helper for making a request to the weather API.
    async fn request<'de, T: Deserialize<'de>>(&self, source: ApiSource, request: Request) -> T {
        let url = Url::parse_with_params(&source.to_string(), request.into_iter()).unwrap();

        // Make the response
        let response = self.client.get(url).send().await.unwrap();

        // Extract the body
        let mut body = response.json::<Value>().await.unwrap();

        // Extract the 'daily' key and deserialise the value
        T::deserialize(body["daily"].take()).unwrap()
    }
}

pub enum ApiSource {
    Forecast,
    Archive,
}

impl Display for ApiSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ApiSource::Forecast => "https://api.open-meteo.com/v1/forecast",
                ApiSource::Archive => "https://archive-api.open-meteo.com/v1/archive",
            }
        )
    }
}

pub struct Request {
    start_date: NaiveDate,
    end_date: NaiveDate,
    latitude: f64,
    longitude: f64,
    parameters: &'static [&'static str],
}

impl IntoIterator for Request {
    type Item = (&'static str, String);

    type IntoIter = std::array::IntoIter<Self::Item, 6>;

    fn into_iter(self) -> Self::IntoIter {
        [
            ("start_date", self.start_date.to_string()),
            ("end_date", self.end_date.to_string()),
            ("latitude", self.latitude.to_string()),
            ("longitude", self.longitude.to_string()),
            ("timezone", "auto".to_string()),
            ("daily", self.parameters.join(",")),
        ]
        .into_iter()
    }
}
