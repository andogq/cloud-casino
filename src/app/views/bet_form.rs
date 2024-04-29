use maud::{html, Markup};
use serde::Deserialize;
use time::Date;

use crate::app::services::bet::Bet;

fn input(
    name: impl AsRef<str>,
    label: impl AsRef<str>,
    icon: impl AsRef<str>,
    value: impl AsRef<str>,
    after: Option<impl AsRef<str>>,
) -> Markup {
    html! {
        label .icon-input {
            p .label { (label.as_ref()) }

            .pill {
                i data-lucide=(icon.as_ref()) {}
                input type="text" name=(name.as_ref()) value=(value.as_ref());

                @if let Some(after) = after {
                    span { (after.as_ref()) }
                }
            }
        }
    }
}

fn rain_button(
    label: impl AsRef<str>,
    icon: impl AsRef<str>,
    value: bool,
    checked: bool,
) -> Markup {
    html! {
        label {
            input name="rain" value=(value) type="radio" checked[checked];
            i data-lucide=(icon.as_ref()) {}
            span { (label.as_ref()) }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BetForm {
    pub rain: bool,
    pub temperature: f64,
    pub range: f64,
    pub wager: f64,
}

impl From<BetForm> for Bet {
    fn from(form: BetForm) -> Self {
        Self::from(&form)
    }
}

impl From<&BetForm> for Bet {
    fn from(form: &BetForm) -> Self {
        Self {
            temperature: form.temperature,
            range: form.range,
            rain: form.rain,
            wager: form.wager,
        }
    }
}

impl From<Bet> for BetForm {
    fn from(bet: Bet) -> Self {
        Self::from(&bet)
    }
}

impl From<&Bet> for BetForm {
    fn from(bet: &Bet) -> Self {
        Self {
            temperature: bet.temperature,
            range: bet.range,
            rain: bet.rain,
            wager: bet.wager,
        }
    }
}

pub fn render_maximum_payout(date: Date, payout: f64) -> Markup {
    html! {
        p #maximum-payout
            hx-get=(format!("/app/bet/{date}/payout")) hx-trigger="input from:closest form" hx-include="#bet-form input"
        {
            "maximum payout: "
            (format!("${payout:.2}"))
        }
    }
}

pub fn render(date: Date, value: BetForm, maximum_payout: f64) -> Markup {
    html! {
        form #bet-form .peek
            action=(format!("/app/bet/{date}")) method="post"
            hx-boost="true" hx-disabled-elt="this"
        {
            #rain-guess .pill {
                (rain_button("sunny", "sun", true, value.rain == true))
                (rain_button("rainy", "cloud-rain", false, value.rain == false))
            }

            #temperature {
                (input("temperature", "temperature?", "thermometer", value.temperature.to_string(), Some("°")))
                (input("range", "range?", "diff", value.range.to_string(), Some("°")))
            }

            (input("wager", "wager?", "badge-dollar-sign", value.wager.to_string(), Option::<&str>::None))

            (render_maximum_payout(date, maximum_payout))

            button type="submit" #bet-button { "place bet" }
        }
    }
}
