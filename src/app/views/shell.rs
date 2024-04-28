use maud::{html, Markup};

use crate::app::views;

pub fn render() -> Markup {
    let balance = 183.40;

    html! {
        main {
            (views::head::render(balance))

            #draw {
                (views::forecast::render())

                (views::bet_form::render())

                (views::payout::render_pill())
            }
        }
    }
}
