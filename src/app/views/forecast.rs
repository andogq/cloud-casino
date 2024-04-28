use maud::{html, Markup};
use time::{macros::format_description, Date};

pub struct Forecast {
    pub date: Date,
    pub rain: f64,
    pub min: f64,
    pub max: f64,
}

pub fn render(days: Vec<Forecast>) -> Markup {
    html! {
        #forecast {
            h2 { "forecast" }

            .days {
                @for day in days {
                    .weather-tile {
                        p .day { (day.date.format(format_description!("[weekday repr:short]")).unwrap()) }

                        .icon .invert {
                            i data-lucide="cloud-sun" {}
                        }

                        .line .rain {
                            p { "25%" }
                            i data-lucide="droplets" {}
                        }

                        .line .temperature {
                            p { "16° / 23°" }
                            i data-lucide="thermometer" {}
                        }
                    }
                }
            }
        }
    }
}
