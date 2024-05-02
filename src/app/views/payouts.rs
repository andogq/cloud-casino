use maud::{html, Markup};
use time::{macros::format_description, Date, OffsetDateTime};

use crate::app::services::bet::Bet;

pub struct Payout {
    /// Date this payout is for.
    date: Date,

    /// The bet that was placed.
    bet: Bet,

    /// Whether rain was experienced on the day.
    rain: bool,

    /// Whether the rain guess was correct
    rain_correct: bool,

    /// The average temperature of the day.
    temperature: f64,

    /// Whether the temperature guess was correct
    temperature_correct: bool,

    /// The final payout for this day.
    payout: f64,
}

fn rain_icon(rain: bool) -> Markup {
    html! {
        i data-lucide=(if rain { "cloud-rain" } else { "sun"}) {}
    }
}

pub fn render() -> Markup {
    let payouts = vec![
        Payout {
            date: OffsetDateTime::now_utc().date(),
            bet: Bet {
                temperature: 16.0,
                range: 3.0,
                rain: false,
                wager: 100.0,
            },

            rain: false,
            rain_correct: true,

            temperature: 21.0,
            temperature_correct: false,

            payout: 50.0,
        },
        Payout {
            date: OffsetDateTime::now_utc().date().next_day().unwrap(),
            bet: Bet {
                temperature: 16.0,
                range: 3.0,
                rain: false,
                wager: 100.0,
            },

            rain: true,
            rain_correct: false,

            temperature: 16.0,
            temperature_correct: true,

            payout: 150.0,
        },
        Payout {
            date: OffsetDateTime::now_utc().date(),
            bet: Bet {
                temperature: 16.0,
                range: 3.0,
                rain: false,
                wager: 100.0,
            },

            rain: false,
            rain_correct: true,

            temperature: 21.0,
            temperature_correct: false,

            payout: 50.0,
        },
        Payout {
            date: OffsetDateTime::now_utc().date().next_day().unwrap(),
            bet: Bet {
                temperature: 16.0,
                range: 3.0,
                rain: false,
                wager: 100.0,
            },

            rain: true,
            rain_correct: false,

            temperature: 16.0,
            temperature_correct: true,

            payout: 150.0,
        },
        Payout {
            date: OffsetDateTime::now_utc().date(),
            bet: Bet {
                temperature: 16.0,
                range: 3.0,
                rain: false,
                wager: 100.0,
            },

            rain: false,
            rain_correct: true,

            temperature: 21.0,
            temperature_correct: false,

            payout: 50.0,
        },
        Payout {
            date: OffsetDateTime::now_utc().date().next_day().unwrap(),
            bet: Bet {
                temperature: 16.0,
                range: 3.0,
                rain: false,
                wager: 100.0,
            },

            rain: true,
            rain_correct: false,

            temperature: 16.0,
            temperature_correct: true,

            payout: 150.0,
        },
        Payout {
            date: OffsetDateTime::now_utc().date(),
            bet: Bet {
                temperature: 16.0,
                range: 3.0,
                rain: false,
                wager: 100.0,
            },

            rain: false,
            rain_correct: true,

            temperature: 21.0,
            temperature_correct: false,

            payout: 50.0,
        },
        Payout {
            date: OffsetDateTime::now_utc().date().next_day().unwrap(),
            bet: Bet {
                temperature: 16.0,
                range: 3.0,
                rain: false,
                wager: 100.0,
            },

            rain: true,
            rain_correct: false,

            temperature: 16.0,
            temperature_correct: true,

            payout: 150.0,
        },
    ];

    let payout_total = payouts.iter().map(|p| p.payout).sum::<f64>();

    html! {
        .peek {
            #payouts {
                @for payout in payouts {
                    .pill {
                        .date { (payout.date.format(format_description!("[weekday repr:short], [month repr:long] [day padding:none] [year]")).unwrap().to_lowercase()) }

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

            button { "payout " (format!("${:.2}", payout_total)) }
        }
    }
}
