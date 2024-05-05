use maud::{html, Markup};

use crate::app::views;

pub fn render(
    balance: f64,
    payout_count: usize,
    show_payout: bool,
    draw_content: Markup,
) -> Markup {
    html! {
        main {
            (views::head::render(balance, payout_count, show_payout))

            #draw {
                (draw_content)
            }
        }
    }
}
