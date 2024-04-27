use maud::{html, Markup};

use crate::user::User;

pub async fn render(user: &User) -> Markup {
    html! {
        h1 style="grid-area: title" { "Place Your Bets" }

        label style="grid-area: temperature" {
            "Temperature: "
            br;
            input type="number" name="temperature" value="20";
        }

        label style="grid-area: rain" {
            "Will it rain?"
            input type="checkbox" name="rain";
        }

        label style="grid-area: confidence" x-data="{ value: '0' }" {
            "Confidence: "
            span x-text="value" {}
            "%"

            br;

            input type="range" name="confidence" min="0" max="100" step="5" value="50" x-model="value";
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
