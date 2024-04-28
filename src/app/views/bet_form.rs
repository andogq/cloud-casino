use maud::{html, Markup};

fn input(icon: &str, value: &str, after: Option<&str>) -> Markup {
    html! {
        label .icon-input .pill {
            i data-lucide=(icon) {}
            input type="text" value=(value);

            @if let Some(after) = after {
                span { (after) }
            }
        }
    }
}

pub fn render() -> Markup {
    html! {
        #bet-form .peek {
            #rain-guess .pill {
                @for icon in ["cloud-rain", "sun"] {
                    label {
                        input name="rain" type="radio";
                        i data-lucide=(icon) {}
                    }
                }
            }

            #temperatures {
                (input("thermometer-snowflake", "16", Some("°")))
                (input("thermometer-sun", "23", Some("°")))
            }

            (input("badge-dollar-sign", "20.00", None))

            button #bet-button .arrow { "bet" }
        }
    }
}
