use reqwest::Client;
use sqlx::SqlitePool;

use self::{bet::BetService, weather::WeatherService};

pub mod bet;
pub mod new_weather;
pub mod weather;

#[derive(Clone)]
pub struct Services {
    pub bet: BetService,
    pub weather: WeatherService,
    pub new_weather: new_weather::WeatherService,
}

impl Services {
    pub fn new(pool: SqlitePool, client: Client) -> Self {
        let weather = WeatherService::new();

        Self {
            bet: BetService::new(pool.clone(), weather.clone()),
            weather,
            new_weather: new_weather::WeatherService::new(pool, client),
        }
    }
}
