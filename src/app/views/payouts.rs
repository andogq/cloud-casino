use chrono::NaiveDate;
use maud::{html, Markup};

use crate::services::bet::Bet;

pub struct Payout {
    /// Date this payout is for.
    pub date: NaiveDate,

    /// The bet that was placed.
    pub bet: Bet,

    /// Whether rain was experienced on the day.
    pub rain: bool,

    /// Whether the rain guess was correct
    pub rain_correct: bool,

    /// The average temperature of the day.
    pub temperature: f64,

    /// Whether the temperature guess was correct
    pub temperature_correct: bool,

    /// The final payout for this day.
    pub payout: f64,
}

fn rain_icon(rain: bool) -> Markup {
    html! {
        i data-lucide=(if rain { "cloud-rain" } else { "sun"}) {}
    }
}

pub fn render(payouts: &[Payout]) -> Markup {
    let payout_total = payouts.iter().map(|p| p.payout).sum::<f64>();

    html! {
        .peek {
            #payouts {
                @for payout in payouts {
                    .pill {
                        .date { (payout.date.format("%a, %B %e %Y").to_string().to_lowercase()) }

                        .bet-rain { (rain_icon(payout.bet.rain)) }

                        .bet-temperature {
                            i data-lucide="thermometer" {}
                            span { (payout.bet.temperature) "°" }
                        }

                        .bet-range {
                            i data-lucide="diff" {}
                            span { (payout.bet.range) "°" }
                        }

                        .arrow .faded {
                            i data-lucide="arrow-right" {}
                        }

                        .actual-rain .correct[payout.rain_correct] .incorrect[!payout.rain_correct] {
                            (rain_icon(payout.rain))
                        }

                        .actual-temperature .correct[payout.temperature_correct] .incorrect[!payout.temperature_correct] {
                            i data-lucide="thermometer" {}
                            span { (payout.temperature) "°" }
                        }

                        .line .faded {}

                        .payout {
                            p { (format!("${:.2}", payout.payout)) }
                        }
                    }
                }
            }

            button hx-post="/payout" hx-trigger="click" {
                "payout " (format!("${:.2}", payout_total))
            }
        }
    }
}
