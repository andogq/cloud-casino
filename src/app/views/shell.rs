use maud::{html, Markup};
use time::Date;

use crate::{app::views, weather::Forecast};

pub fn render(balance: f64, payout_count: usize, draw_content: Markup) -> Markup {
    html! {
        main {
            (views::head::render(balance, payout_count))

            #draw {
                (draw_content)
            }
        }
    }
}
