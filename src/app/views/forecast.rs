use maud::{html, Markup};

pub fn render() -> Markup {
    html! {
        #forecast {
            h2 { "forecast" }

            .days {
                @for _ in 0..7 {
                    .weather-tile {
                        p .day { "mon" }

                        .icon .invert {
                            i data-lucide="cloud-sun" {}
                        }

                        .line .rain {
                            p { "25%" }
                            i data-lucide="droplets" {}
                        }

                        .line .temperature {
                            p { "16° / 23°" }
                            i data-lucide="thermometer" {}
                        }
                    }
                }
            }
        }
    }
}
