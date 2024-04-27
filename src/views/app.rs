use maud::{html, Markup};

use crate::user::User;

pub async fn render(user: &User) -> Markup {
    html! {
        .card {
            h1 { "Summary" }
            ul {
                li { "Balance: " (format!("${:.2}", user.data.balance)) }
                li { "Outstanding Bets: " (user.data.outstanding_bets.len()) }
            }
        }
    }
}
