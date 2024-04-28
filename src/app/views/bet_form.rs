use maud::{html, Markup};

fn input(name: &str, icon: &str, value: &str, after: Option<&str>) -> Markup {
    html! {
        label .icon-input .pill {
            i data-lucide=(icon) {}
            input type="text" name=(name) value=(value);

            @if let Some(after) = after {
                span { (after) }
            }
        }
    }
}

pub struct BetFormValue {
    pub rain: bool,
    pub min_temp: f64,
    pub max_temp: f64,
    pub wager: f64,
}

pub fn render(prefill: Option<BetFormValue>) -> Markup {
    html! {
        #bet-form .peek {
            #rain-guess .pill {
                @for (icon, value) in [("cloud-rain", true), ("sun", false)] {
                    label {
                        input name="rain" value=(value) type="radio";
                        i data-lucide=(icon) {}
                    }
                }
            }

            #temperatures {
                (input("min_temp", "thermometer-snowflake", &prefill.as_ref().map(|value| value.min_temp.to_string()).unwrap_or_default(), Some("°")))
                (input("max_temp", "thermometer-sun", &prefill.as_ref().map(|value| value.max_temp.to_string()).unwrap_or_default(), Some("°")))
            }

            (input("wager", "badge-dollar-sign", &prefill.as_ref().map(|value| value.wager.to_string()).unwrap_or_default(), None))

            button #bet-button .arrow { "bet" }
        }
    }
}
