use self::{bet::BetService, weather::WeatherService};

pub mod bet;
pub mod weather;

#[derive(Clone)]
pub struct Services {
    pub bet: BetService,
    pub weather: WeatherService,
}

impl Services {
    pub fn new() -> Self {
        let weather = WeatherService::new();

        Self {
            bet: BetService::new(weather.clone()),
            weather,
        }
    }
}
