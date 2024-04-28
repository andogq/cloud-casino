use maud::{html, Markup};
use serde::Deserialize;
use time::Date;

fn input(
    name: impl AsRef<str>,
    icon: impl AsRef<str>,
    value: impl AsRef<str>,
    after: Option<impl AsRef<str>>,
) -> Markup {
    html! {
        label .icon-input .pill {
            i data-lucide=(icon.as_ref()) {}
            input type="text" name=(name.as_ref()) value=(value.as_ref());

            @if let Some(after) = after {
                span { (after.as_ref()) }
            }
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

pub fn render(date: Date, prefill: Option<BetFormValue>) -> Markup {
    // If no prefill is provided, just use some default values
    let value = prefill.unwrap_or_else(|| BetFormValue {
        rain: false,
        min_temp: 18.0,
        max_temp: 25.0,
        wager: 10.0,
    });

    html! {
        form #bet-form .peek
            action=(format!("/app/bet/{date}")) method="post"
            hx-boost="true" hx-disabled-elt="this"
        {
            #rain-guess .pill {
                @for (icon, state) in [("cloud-rain", true), ("sun", false)] {
                    label {
                        input name="rain" value=(state) type="radio" checked[state == value.rain];
                        i data-lucide=(icon) {}
                    }
                }
            }

            #temperatures {
                (input("min_temp", "thermometer-snowflake", value.min_temp.to_string(), Some("°")))
                (input("max_temp", "thermometer-sun", value.max_temp.to_string(), Some("°")))
            }

            (input("wager", "badge-dollar-sign", value.wager.to_string(), Option::<&str>::None))

            button type="submit" #bet-button .arrow { "bet" }
        }
    }
}
