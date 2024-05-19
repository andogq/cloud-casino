use maud::{html, Markup};

pub fn render(hero: String, payout_count: usize, show_payout: bool) -> Markup {
    html! {
        #head {
            h1 #hero { (hero) }

            @if payout_count > 0 && show_payout {
                a href="/payout" hx-boost="true" #payout {
                    p {
                        span .count { (payout_count) }

                        " "

                        @if payout_count == 1 {
                            "payout"
                        } @else {
                            "payouts"
                        }

                        " ready"
                    }

                    i data-lucide="chevron-right" {}
                }
            }
        }
    }
}
