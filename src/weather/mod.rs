mod forecast;
mod service;

use chrono::NaiveDate;

pub use forecast::render_forecast;
pub use service::Service;

#[derive(Debug, Clone)]
pub struct Forecast {
    pub date: NaiveDate,
    pub rain: f64,
    pub min: f64,
    pub max: f64,
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
