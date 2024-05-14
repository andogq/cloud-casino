use reqwest::Client;
use sqlx::SqlitePool;

use self::{bet::BetService, oauth::OAuthService, state::StateService, weather::WeatherService};

pub mod bet;
pub mod oauth;
pub mod state;
pub mod weather;

#[derive(Clone)]
pub struct Services {
    pub bet: BetService,
    pub weather: WeatherService,
    pub oauth: OAuthService,
    pub state: StateService,
}

impl Services {
    pub fn new(pool: SqlitePool, client: Client) -> Self {
        let weather = WeatherService::new(pool.clone(), client.clone());
        let state = StateService::new(pool.clone());

        Self {
            bet: BetService::new(pool.clone(), weather.clone()),
            oauth: OAuthService::new(pool, client.clone(), state.clone()),
            weather,
            state,
        }
    }
}
