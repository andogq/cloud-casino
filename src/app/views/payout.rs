use maud::{html, Markup};

pub fn render_pill() -> Markup {
    html! {
        #payout .peek {
            .count {
                h3 { "4" }
            }

            h2 .arrow { "payouts ready" }
        }
    }
}
