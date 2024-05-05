use maud::{html, Markup};
use time::{macros::format_description, Date};

use crate::weather::Forecast;

fn pick_wmo_icon(code: usize) -> &'static str {
    match code {
        0 | 1 => "sun",
        2 => "cloud-sun",
        3 => "cloud",
        45 | 48 => "cloud-fog",
        51 | 53 | 55 | 56 | 57 | 80 | 81 | 82 => "cloud-drizzle",
        61 | 63 | 65 | 66 | 67 => "cloud-rain",
        71 | 73 | 75 | 77 | 85 | 86 => "snowflake",
        95 | 96 | 99 => "cloud-lightning",
        _ => "circle-help",
    }
}

pub fn render(days: Vec<(Forecast, f64)>, selected: Option<Date>) -> Markup {
    html! {
        form #forecast hx-get="/bet" hx-trigger="change" hx-target="#bet-form" hx-swap="outerHTML" {
            .days {
                label .deselect {
                    input type="radio" name="date" value="null" checked[selected.is_none()];
                }

                @for (day, bet_placed) in days {
                    @let checked = selected.map(|date| date == day.date).unwrap_or(false);
                    label .weather-tile {
                        input type="radio" name="date" autocomplete="off"
                            value=(day.date) checked[checked];

                        p .day {
                            (day.date.format(format_description!("[weekday repr:short]")).unwrap())
                        }

                        i data-lucide=(pick_wmo_icon(day.wmo_code)) {}

                        .line .rain {
                            p { (format!("{:.0}%", day.rain * 100.0)) }
                            i data-lucide="droplets" {}
                        }

                        .line .temperature {
                            p { (format!("{:.0}° / {:.0}°", day.min, day.max)) }
                            i data-lucide="thermometer" {}
                        }

                        .line .bet-amount {
                            p { (format!("${bet_placed:.2}"))}
                            i data-lucide="badge-dollar-sign" {}
                        }
                    }
                }
            }
        }
    }
}
