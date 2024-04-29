use maud::{html, Markup};
use time::OffsetDateTime;

use crate::{app::views, weather::Forecast};

use super::bet_form::BetForm;

pub fn render(
    balance: f64,
    forecast: Vec<Forecast>,
    form_value: BetForm,
    maximum_payout: f64,
    payout_count: usize,
) -> Markup {
    html! {
        main {
            (views::head::render(balance))

            #draw {
                (views::forecast::render(forecast))

                (views::bet_form::render(OffsetDateTime::now_utc().date(), form_value, maximum_payout))

                (views::payout::render_pill(payout_count))
            }
        }
    }
}
