use maud::{html, Markup};
use time::macros::format_description;

use crate::{
    user::User,
    weather::{Point, WeatherService},
};

pub async fn render(user: &User, service: WeatherService, location: Point) -> Markup {
    let forecast = service.get_forecast(location).await;

    html! {
        #forecast.card {
            h1 { "Weekly Forecast" }

            button hx-get="/forecast" hx-target="#forecast" hx-swap="outerHTML" { "Refresh" }

            div style="display: flex; flex-direction: row; justify-content: space-between;" {
                @for day in forecast {
                    label
                        .day
                        .outstanding[user.data.outstanding_bets.contains(&day.date)]
                    {
                        p.date { (day.date.format(format_description!("[day]/[month]")).unwrap()) }
                        p.temperature { (day.min) "°C / " (day.max) "°C" }
                        p.rain { ((day.rain * 100.0).round() as isize) "% rain" }

                        input type="radio" name="day" value=(day.date);
                    }
                }
            }
        }
    }
}
