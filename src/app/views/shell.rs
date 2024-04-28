use maud::{html, Markup};

use crate::{app::views, weather::Forecast};

use super::bet_form::BetFormValue;

pub fn render(
    balance: f64,
    forecast: Vec<Forecast>,
    prefill: Option<BetFormValue>,
    payout_count: usize,
) -> Markup {
    html! {
        main {
            (views::head::render(balance))

            #draw {
                (views::forecast::render(forecast))

                (views::bet_form::render(prefill))

                (views::payout::render_pill(payout_count))
            }
        }
    }
}
