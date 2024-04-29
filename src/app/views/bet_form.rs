use maud::{html, Markup};
use serde::Deserialize;
use time::Date;

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
pub struct BetFormValue {
    pub rain: bool,
    pub min_temp: f64,
    pub max_temp: f64,
    pub wager: f64,
}

pub fn render(date: Date, value: BetFormValue, maximum_payout: f64) -> Markup {
    html! {
        form #bet-form .peek
            action=(format!("/app/bet/{date}")) method="post"
            hx-boost="true" hx-disabled-elt="this"
        {
            #rain-guess .pill {
                (rain_button("sunny", "sun", true, value.rain == true))
                (rain_button("rainy", "cloud-rain", false, value.rain == false))
            }

            #temperatures {
                (input("min_temp", "min temp?", "thermometer-snowflake", value.min_temp.to_string(), Some("°")))
                (input("max_temp", "max temp?", "thermometer-sun", value.max_temp.to_string(), Some("°")))
            }

            (input("wager", "wager?", "badge-dollar-sign", value.wager.to_string(), Option::<&str>::None))

            p #maximum-payout {
                "maximum payout: "
                (format!("${maximum_payout:.2}"))
            }

            button type="submit" #bet-button { "place bet" }
        }
    }
}
