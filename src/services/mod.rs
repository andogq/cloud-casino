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
    pub fn new(pool: SqlitePool) -> Self {
        let weather = WeatherService::new();

        Self {
            bet: BetService::new(pool, weather.clone()),
            weather,
        }
    }
}
