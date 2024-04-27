use maud::{html, Markup};
use time::OffsetDateTime;

use crate::user::User;

pub async fn render(user: &User) -> Markup {
    let payout_ready = user
        .data
        .outstanding_bets
        .iter()
        .filter(|day| day < &&OffsetDateTime::now_utc().date())
        .count();

    html! {
        .card {
            h1 { "Summary" }
            ul {
                li { "Balance: " (format!("${:.2}", user.data.balance)) }
                li { "Outstanding Bets: " (user.data.outstanding_bets.len()) }
                li {
                    "Ready for Payout: " (payout_ready)
                    @if payout_ready > 0 {
                        " "
                        a href="/payout" hx-boost="true" { "Payout now" }
                    }
                }
            }
        }
    }
}
