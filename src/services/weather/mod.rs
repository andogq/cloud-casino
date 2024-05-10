mod api;
mod db;

use chrono::{Duration, NaiveDate, Utc};
use num_enum::{FromPrimitive, IntoPrimitive};
use reqwest::Client;
use sqlx::SqlitePool;

use crate::MELBOURNE;

use self::{api::Api, db::Db};

#[derive(Clone)]
pub struct WeatherService {
    api: Api,
    db: Db,
}

impl WeatherService {
    /// Create a new instance of the weather service.
    pub fn new(pool: SqlitePool, client: Client) -> Self {
        Self {
            api: Api::new(client),
            db: Db::new(pool),
        }
    }

    /// Get the forecast for the provided date. Given that forecasts change over time, only one
    /// forecast is generated per day.
    pub async fn get_daily_forecast(&self, date: NaiveDate, location: (f64, f64)) -> Forecast {
        let now = chrono::offset::Utc::now();

        // Check if a forecast already exists for this date
        if let Some(forecast) = self.db.get_day_forecast(date, now).await {
            forecast
        } else {
            // Fetch the forecast from the weather API
            let forecast = self.api.get_daily_forecast(date, location).await;

            // Save it into the DB
            self.db.save_forecast(date, now, &forecast).await;

            // Return the forecast
            forecast
        }
    }

    /// Get the forecast for some date range
    pub async fn get_forecast(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Vec<(NaiveDate, Forecast)> {
        let now = chrono::offset::Utc::now();

        // Fetch the saved forecast for this date range
        let mut forecast = self.db.get_forecast_range(start, end, now).await;

        // If all the days are present, then no need to continue
        let days_inclusive = (end - start).num_days().abs() as usize + 1;
        if forecast.len() == days_inclusive {
            return forecast;
        }

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
            if &filter_end == date && filter_end > filter_start {
                filter_end -= Duration::days(1);
            } else {
                // This day is missing, filter must end here
                break;
            }
        }

        // Get the missing days, and add them to the forecast
        for (date, day_forecast) in self
            .api
            .get_forecast(filter_start, filter_end, MELBOURNE)
            .await
        {
            // See if the day is already in the provided forecast
            if forecast.iter().find(|d| d.0 == date).is_some() {
                continue;
            }

            // Save the collection in the DB
            self.db.save_forecast(date, now, &day_forecast).await;

            // Add the forecast to the collection
            forecast.push((date, day_forecast));
        }

        // Sort the forecast by the date again
        forecast.sort_unstable_by_key(|&(date, _)| date);

        forecast
    }

    pub async fn get_historical_weather(&self, date: NaiveDate) -> Option<Weather> {
        // Check if it's in the DB
        if let Some(weather) = self.db.get_historical_weather(date).await {
            return Some(weather);
        }

        // Get the weather from the API
        let (_, weather) = self.api.get_historical(date, date, MELBOURNE).await.pop()?;

        // Save it in the DB for later
        self.db
            .save_historical_weather(date, Utc::now(), &weather)
            .await;

        Some(weather)
    }
}

#[derive(Clone, Debug)]
pub struct Forecast {
    pub rain: f64,
    pub minimum_temperature: f64,
    pub maximum_temperature: f64,
    pub weather_code: WeatherCode,
}

#[derive(Clone, Debug)]
pub struct Weather {
    pub rain: bool,
    pub temperature: f64,
}

#[derive(Clone, Copy, Debug, IntoPrimitive, FromPrimitive, sqlx::Type)]
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
