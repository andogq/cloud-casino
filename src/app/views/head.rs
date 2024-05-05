use maud::{html, Markup};

pub fn render(balance: f64, payout_count: usize, show_payout: bool) -> Markup {
    html! {
        #head {
            h1 #balance { (format!("${balance:.2}")) }

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
