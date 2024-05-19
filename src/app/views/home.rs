use maud::{html, Markup};

use super::login::Provider;

pub fn render(providers: Option<&[Provider]>) -> Markup {
    html! {
        #home .bet-form-target {
            p { "Bet (fake) money on the weather!" }

            @if let Some(providers) = providers {
                p { "To begin, log in with a provider below" }

                @for provider in providers {
                    a .button href=(provider.url) {
                        i data-lucide=(provider.icon) {}

                        span { (provider.name) }
                    }
                }
            }

            p {
                "Place a bet by selecting a day (other than today) from the forecast. Place your guess
                on whether it will rain ('rainy') or not ('sunny'), the average temperature
                ('temperature'), and a range for the average temperature to fall between. Finally,
                place your wager! The maximum possible payout will be shown underneath."
            }

            h2 { "payout calculation" }

            p {
                "The further in the future the selected date is, the larger the payout multiplier is.
                Payouts are split into two components, one for the rain prediction, and one for the
                temperature prediction. If the rain prediction is correct, a payout will be awarded. If
                the difference between a day's average temperature and the bet's temperature is less
                than the bet range, a payout will also be awarded!"
            }

            p {
                "Payouts will be available from 10am the following day."
            }

            p {
                "made by "
                a href="https://ando.gq" target="_blank" { "ando.gq" }
            }
        }
    }
}
