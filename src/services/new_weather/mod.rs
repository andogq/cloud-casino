mod api;

use chrono::{Duration, NaiveDate};
use num_enum::{FromPrimitive, IntoPrimitive};
use reqwest::Client;
use sqlx::SqlitePool;

use crate::MELBOURNE;

use self::api::Api;

#[derive(Clone)]
pub struct WeatherService {
    pool: SqlitePool,
    api: Api,
}

impl WeatherService {
    /// Create a new instance of the weather service.
    pub fn new(pool: SqlitePool, client: Client) -> Self {
        Self {
            pool,
            api: Api::new(client),
        }
    }

    /// Get the forecast for the provided date. Given that forecasts change over time, only one
    /// forecast is generated per day.
    pub async fn get_daily_forecast(&self, date: NaiveDate, location: (f64, f64)) -> Forecast {
        let now = chrono::offset::Utc::now();

        // Check if a forecast already exists for this date
        if let Some(forecast) = sqlx::query_as!(
            Forecast,
            "SELECT rain, minimum_temperature, maximum_temperature, weather_code
                FROM forecasts
                WHERE date = ? AND date_retrieved = ?;",
            date,
            now
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap()
        {
            forecast
        } else {
            // Fetch the forecast from the weather API
            let forecast = self.api.get_daily_forecast(date, location).await;
            let weather_code = forecast.weather_code as i64;

            // Save it into the DB
            sqlx::query!(
                "INSERT INTO forecasts (date, date_retrieved, rain, minimum_temperature, maximum_temperature, weather_code)
                    VALUES (?, ?, ?, ?, ?, ?);",
                date,
                now,
                forecast.rain,
                forecast.minimum_temperature,
                forecast.maximum_temperature,
                weather_code,
            )
                .execute(&self.pool)
                .await
                .unwrap();

            // Return the forecast
            forecast
        }
    }

    /// Get the forecast for some date range
    pub async fn get_forecast(&self, start: NaiveDate, end: NaiveDate) -> Vec<Forecast> {
        let now = chrono::offset::Utc::now();

        struct Result {
            date: NaiveDate,
            rain: bool,
            minimum_temperature: f64,
            maximum_temperature: f64,
            weather_code: WeatherCode,
        }

        let mut forecast = sqlx::query_as!(
            Result,
            "SELECT date, rain, minimum_temperature, maximum_temperature, weather_code
                FROM forecasts
                WHERE date >= ? AND date <= ? AND date_retrieved = ?
                ORDER BY date;",
            start,
            end,
            now
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
        .unwrap();

        // Find the first missing date
        let mut filter_start = start;
        for (date, _) in &forecast {
            if &filter_start == date {
                filter_start += Duration::days(1);
            } else {
                // This day is missing, filter must start here
                break;
            }
        }

        let mut filter_end = end;
        for (date, _) in forecast.iter().rev() {
            if &filter_end == date {
                filter_end += Duration::days(1);
            } else {
                // This day is missing, filter must end here
                break;
            }
        }

        // Get the missing days, and add them to the forecast
        for (date, day_forecast) in self
            .api
            .get_forecast(
                filter_start,
                filter_end,
                (MELBOURNE.latitude, MELBOURNE.longitude),
            )
            .await
        {
            // See if the day is already in the provided forecast
            if forecast.iter().find(|d| d.0 == date).is_some() {
                continue;
            }

            // Add the forecast
            forecast.push((date, day_forecast));
        }

        // Sort the forecast by the date again
        forecast.sort_unstable_by_key(|&(date, _)| date);

        forecast.into_iter().map(|(_, forecast)| forecast).collect()
    }
}

#[derive(Clone, Debug)]
pub struct Forecast {
    pub rain: bool,
    pub minimum_temperature: f64,
    pub maximum_temperature: f64,
    pub weather_code: WeatherCode,
}

#[derive(Clone, Copy, Debug, IntoPrimitive, FromPrimitive)]
#[repr(i64)]
pub enum WeatherCode {
    #[num_enum(alternatives = [1])]
    Sun = 0,
    PartialSun = 2,
    Cloud = 3,
    #[num_enum(alternatives = [48])]
    Fog = 45,
    #[num_enum(alternatives = [53, 55, 56, 57, 80, 81, 82])]
    Drizzle = 51,
    #[num_enum(alternatives = [63, 65, 66, 67])]
    Rain = 61,
    #[num_enum(alternatives = [73, 75, 77, 85, 86])]
    Snow = 71,
    #[num_enum(alternatives = [96, 99])]
    Lightning = 95,
    #[num_enum(default)]
    Unknown = -1,
}
