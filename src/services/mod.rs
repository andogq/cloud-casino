use reqwest::Client;
use sqlx::SqlitePool;

use self::{bet::BetService, weather::WeatherService};

pub mod bet;
pub mod weather;

#[derive(Clone)]
pub struct Services {
    pub bet: BetService,
    pub weather: WeatherService,
}

impl Services {
    pub fn new(pool: SqlitePool, client: Client) -> Self {
        let weather = WeatherService::new(pool.clone(), client.clone());

        Self {
            bet: BetService::new(pool, weather.clone()),
            weather,
        }
    }
}
