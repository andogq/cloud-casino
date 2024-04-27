use maud::{html, Markup};

use crate::user::User;

pub async fn render(user: &User) -> Markup {
    html! {
        h1 style="grid-area: title" { "Place Your Bets" }

        label style="grid-area: min" {
            "Minimum Temperature: "
            br;
            input type="number" name="min" value="20";
        }

        label style="grid-area: max" {
            "Maximum Temperature: "
            br;
            input type="number" name="max" value="20";
        }

        label style="grid-area: rain" {
            "Will it rain?"
            input type="checkbox" name="rain";
        }

        label style="grid-area: wager" {
            "Wager: "
            br;
            input type="number" name="wager"
                step="0.01" min="0" max=(user.data.balance)
                value=(user.data.balance * 0.25);
        }

        p style="grid-area: payout; text-align: right" {
            "Maximum potential payout: $"
            span #payout { "0.00" }
        }

        button style="grid-area: submit" type="submit" { "Bet" }
    }
}
