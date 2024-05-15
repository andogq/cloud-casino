use chrono::NaiveDate;
use maud::{html, Markup};
use serde::Deserialize;

use crate::services::bet::Bet;

fn input(
    name: impl AsRef<str>,
    label: impl AsRef<str>,
    icon: impl AsRef<str>,
    value: impl AsRef<str>,
    after: Option<impl AsRef<str>>,
    disabled: bool,
) -> Markup {
    html! {
        label .icon-input {
            p .label { (label.as_ref()) }

            .pill {
                i data-lucide=(icon.as_ref()) {}
                input type="text" name=(name.as_ref()) value=(value.as_ref()) disabled[disabled];

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
    disabled: bool,
) -> Markup {
    html! {
        label {
            input name="rain" value=(value) type="radio" checked[checked] disabled[disabled];
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

pub fn render_maximum_payout(date: NaiveDate, payout: f64) -> Markup {
    html! {
        p #maximum-payout
            hx-get=(format!("/bet/{date}/payout")) hx-trigger="input from:closest form" hx-include="#bet-form input"
        {
            "maximum payout: "
            (format!("${payout:.2}"))
        }
    }
}

pub fn render(
    date: Option<NaiveDate>,
    value: Option<BetForm>,
    maximum_payout: f64,
    replace: bool,
) -> Markup {
    let disabled = value.is_none();

    html! {
        form #bet-form .peek
            autocomplete="off"
            action=[date.map(|date| format!("/bet/{date}"))] method="post"
            hx-boost="true" hx-disabled-elt="#bet-form input, #bet-form button"
        {
            #rain-guess .pill {
                @let sun_value = value.as_ref().map(|value| value.rain == false).unwrap_or(false);
                (rain_button("sunny", "sun", false, sun_value, disabled))

                @let rain_value = value.as_ref().map(|value| value.rain == true).unwrap_or(false);
                (rain_button("rainy", "cloud-rain", true, rain_value, disabled))
            }

            #temperature {
                @let temperature_value = value.as_ref().map(|value| value.temperature.to_string()).unwrap_or_default();
                (input("temperature", "temperature?", "thermometer", temperature_value, Some("°"), disabled))

                @let range_value = value.as_ref().map(|value| value.range.to_string()).unwrap_or_default();
                (input("range", "range?", "diff", range_value, Some("°"), disabled))
            }

            @let wager_value = value.as_ref().map(|value| value.wager.to_string()).unwrap_or_default();
            (input("wager", "wager?", "badge-dollar-sign", wager_value, Option::<&str>::None, disabled))

            @if let Some(date) = date {
                (render_maximum_payout(date, maximum_payout))
            } @else {
                p #maximum-payout {
                    "no payout"
                }
            }

            button type="submit" #bet-button disabled[disabled] {
                @if replace { "re-" }
                "place bet"
            }

            .htmx-indicator {
                .spinner {}
            }
        }
    }
}
