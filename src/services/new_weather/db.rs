use chrono::{DateTime, NaiveDate, Utc};
use sqlx::SqlitePool;

use crate::services::new_weather::WeatherCode;

use super::Forecast;

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get the forecast for a given day, as if it were retrieved on the provied date. `date` must
    /// be in the local timezone of the region the forecast is for.
    pub async fn get_day_forecast(
        &self,
        date: NaiveDate,
        retrieval_date: DateTime<Utc>,
    ) -> Option<Forecast> {
        let retrieval_date = retrieval_date.date_naive();

        sqlx::query_as!(
            Forecast,
            "SELECT rain, minimum_temperature, maximum_temperature, weather_code
                FROM forecasts
                WHERE date = ? AND DATE(date_retrieved) = ?;",
            date,
            retrieval_date
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap()
    }

    /// Load all forecasts between `start_date` and `end_date` that were retrieved on the provided
    /// date. Both `start_date` and `end_date` must be in the local timezone for the region of the
    /// forecast.
    pub async fn get_forecast_range(
        &self,
        start_date: NaiveDate,
        end_date: NaiveDate,
        retrieval_date: DateTime<Utc>,
    ) -> Vec<(NaiveDate, Forecast)> {
        struct Result {
            date: NaiveDate,
            rain: f64,
            minimum_temperature: f64,
            maximum_temperature: f64,
            weather_code: WeatherCode,
        }

        let retrieval_date = retrieval_date.date_naive();

        // Try pull the forecasts for the day that was generated today
        sqlx::query_as!(
            Result,
            "SELECT date, rain, minimum_temperature, maximum_temperature, weather_code
                FROM forecasts
                WHERE date >= ? AND date <= ? AND DATE(date_retrieved) = ?
                ORDER BY date;",
            start_date,
            end_date,
            retrieval_date
        )
        .map(|result| {
            (
                result.date,
                Forecast {
                    rain: result.rain,
                    minimum_temperature: result.minimum_temperature,
                    maximum_temperature: result.maximum_temperature,
                    weather_code: result.weather_code,
                },
            )
        })
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }

    /// Save the forecast for a given day as it was retrived on the provided date.
    pub async fn save_forecast(
        &self,
        date: NaiveDate,
        retrieval_date: DateTime<Utc>,
        forecast: &Forecast,
    ) {
        sqlx::query!(
                "INSERT INTO forecasts (date, date_retrieved, rain, minimum_temperature, maximum_temperature, weather_code)
                    VALUES (?, ?, ?, ?, ?, ?);",
                date,
                retrieval_date,
                forecast.rain,
                forecast.minimum_temperature,
                forecast.maximum_temperature,
                forecast.weather_code,
            )
                .execute(&self.pool)
                .await
                .unwrap();
    }
}
