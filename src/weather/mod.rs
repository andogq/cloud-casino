mod service;

pub use service::WeatherService;
use time::Date;

#[derive(Debug, Clone)]
pub struct Forecast {
    pub date: Date,
    pub rain: f64,
    pub min: f64,
    pub max: f64,
}

pub struct DayWeather {
    pub temperature: f64,
    pub rain: bool,
}

#[derive(Debug, Clone)]
pub struct Point {
    pub latitude: f64,
    pub longitude: f64,
}

impl Point {
    pub fn to_fixed(&self) -> (String, String) {
        (
            format!("{:.3}", self.latitude),
            format!("{:.3}", self.longitude),
        )
    }
}
