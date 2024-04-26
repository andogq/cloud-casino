use maud::{html, Markup};

use super::{Point, Service};

pub async fn render_forecast(service: Service, location: Point) -> Markup {
    let forecast = service.get_forecast(location).await;

    html! {
        #forecast.card {
            h1 { "Weekly Forecast" }

            button hx-get="/forecast" hx-target="#forecast" hx-swap="outerHTML" { "Refresh" }

            div style="display: flex; flex-direction: row; justify-content: space-between;" {
                @for day in forecast {
                    label .day {
                        p.date { (day.date.format("%d/%m")) }
                        p.temperature { (day.min) "°C / " (day.max) "°C" }
                        p.rain { ((day.rain * 100.0).round() as isize) "% rain" }

                        input type="radio" name="day" value=(day.date);
                    }
                }
            }
        }
    }
}
