use chrono::NaiveDate;
use sqlx::SqlitePool;

use crate::services::weather::WeatherCode;

use super::{Forecast, Weather};

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
        retrieval_date: NaiveDate,
    ) -> Option<Forecast> {
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
        retrieval_date: NaiveDate,
    ) -> Vec<(NaiveDate, Forecast)> {
        struct Result {
            date: NaiveDate,
            rain: f64,
            minimum_temperature: f64,
            maximum_temperature: f64,
            weather_code: WeatherCode,
        }

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
    pub async fn save_forecast(&self, date: NaiveDate, forecast: &Forecast) {
        sqlx::query!(
                "INSERT INTO forecasts (date, rain, minimum_temperature, maximum_temperature, weather_code)
                    VALUES (?, ?, ?, ?, ?);",
                date,
                forecast.rain,
                forecast.minimum_temperature,
                forecast.maximum_temperature,
                forecast.weather_code,
            )
                .execute(&self.pool)
                .await
                .unwrap();
    }

    /// Get the historical weather for some date.
    pub async fn get_historical_weather(&self, date: NaiveDate) -> Option<Weather> {
        sqlx::query_as!(
            Weather,
            "SELECT rain, temperature
                FROM historical_weather
                WHERE date = ?",
            date
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap()
    }

    /// Save historical weather for some day as if it were retrieved on the given day.
    pub async fn save_historical_weather(&self, date: NaiveDate, weather: &Weather) {
        sqlx::query!(
            "INSERT INTO historical_weather (date, temperature, rain)
                VALUES (?, ?, ?)",
            date,
            weather.temperature,
            weather.rain,
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }
}
