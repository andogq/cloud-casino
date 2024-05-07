use std::fmt::Display;

use reqwest::{Client, Url};
use serde::Deserialize;
use serde_json::Value;
use time::{format_description::well_known::Iso8601, OffsetDateTime};

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
    pub async fn get_forecast(&self, date: OffsetDateTime, location: (f64, f64)) -> Forecast {
        #[derive(Deserialize)]
        struct ForecastResponse {
            temperature_2m_min: Vec<f64>,
            temperature_2m_max: Vec<f64>,
            precipitation_probability_mean: Vec<f64>,
            weather_code: Vec<i64>,
        }

        let response = self
            .request::<ForecastResponse>(
                ApiSource::Forecast,
                Request {
                    date,
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

        Forecast {
            rain: (response.precipitation_probability_mean[0] / 100.0) > Self::RAIN_THRESHOLD,
            minimum_temperature: response.temperature_2m_min[0],
            maximum_temperature: response.temperature_2m_max[0],
            weather_code: response.weather_code[0].into(),
        }
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
    date: OffsetDateTime,
    latitude: f64,
    longitude: f64,
    parameters: &'static [&'static str],
}

impl IntoIterator for Request {
    type Item = (&'static str, String);

    type IntoIter = std::array::IntoIter<Self::Item, 5>;

    fn into_iter(self) -> Self::IntoIter {
        [
            ("start_date", self.date.format(&Iso8601::DATE).unwrap()),
            ("end_date", self.date.format(&Iso8601::DATE).unwrap()),
            ("latitude", self.latitude.to_string()),
            ("longitude", self.longitude.to_string()),
            ("daily", self.parameters.join(",")),
        ]
        .into_iter()
    }
}
