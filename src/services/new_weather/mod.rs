mod api;

use num_enum::{FromPrimitive, IntoPrimitive};
use reqwest::Client;
use sqlx::SqlitePool;
use time::OffsetDateTime;

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
    pub async fn get_forecast(&self, date: OffsetDateTime) -> Forecast {
        let now = OffsetDateTime::now_utc();

        // Check if a forecast already exists for this date
        if let Some(forecast) = sqlx::query_as!(
            Forecast,
            "SELECT rain, minimum_temperature, maximum_temperature, weather_code FROM forecasts WHERE date = ? AND date_retrieved = ?;",
            date,
            now
        )
            .fetch_optional(&self.pool)
            .await
            .unwrap() {
            forecast
        } else {
            // Fetch the forecast from the weather API
            let forecast = self.api.get_forecast(date, (MELBOURNE.latitude, MELBOURNE.longitude)).await;
            let weather_code = forecast.weather_code as i64;

            // Save it into the DB
            sqlx::query!(
                "INSERT INTO forecasts (date, date_retrieved, rain, minimum_temperature, maximum_temperature, weather_code) VALUES (?, ?, ?, ?, ?, ?);",
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
