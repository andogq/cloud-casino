use maud::{html, Markup};
use time::Date;

use crate::{app::views, weather::Forecast};

pub fn render(
    balance: f64,
    forecast: Vec<Forecast>,
    selected_day: Option<Date>,
    payout_count: usize,
    draw_content: Markup,
) -> Markup {
    html! {
        main {
            (views::head::render(balance, payout_count))

            #draw {
                (views::forecast::render(forecast, selected_day))

                // (views::bet_form::render(selected_day, form_value, maximum_payout))
                // (views::payouts::render())
                (draw_content)
            }
        }
    }
}
