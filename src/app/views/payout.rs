use maud::{html, Markup};

pub fn render_pill(count: usize) -> Markup {
    html! {
        #payout .peek {
            .count {
                h3 { (count) }
            }

            h2 .arrow {
                @if count == 1 {
                    "payout"
                } @else {
                    "payouts"
                }

                " ready"
            }
        }
    }
}
