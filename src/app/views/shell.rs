use maud::{html, Markup};
use time::Date;

use crate::{app::views, weather::Forecast};

use super::bet_form::BetForm;

pub fn render(
    balance: f64,
    forecast: Vec<Forecast>,
    selected_day: Option<Date>,
    form_value: Option<BetForm>,
    maximum_payout: f64,
    payout_count: usize,
) -> Markup {
    html! {
        main {
            (views::head::render(balance, payout_count))

            #draw {
                (views::forecast::render(forecast, selected_day))

                // (views::bet_form::render(selected_day, form_value, maximum_payout))
                (views::payouts::render())
            }
        }
    }
}
