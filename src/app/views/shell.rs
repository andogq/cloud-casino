use maud::{html, Markup};

use crate::app::views;

pub fn render(
    hero: String,
    payout_count: usize,
    show_payout: bool,
    draw_content: Markup,
) -> Markup {
    html! {
        main {
            (views::head::render(hero, payout_count, show_payout))

            #draw {
                (draw_content)
            }
        }
    }
}
