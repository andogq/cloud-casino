use maud::{html, Markup};

pub mod bet_form;
pub mod forecast;
pub mod head;
pub mod login;
pub mod payouts;
pub mod shell;

pub fn page(body: Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                link rel="stylesheet" type="text/css" href="/app.css";
                link rel="stylesheet" type="text/css" href="https://unpkg.com/open-props";
                link rel="stylesheet" type="text/css" href="https://unpkg.com/open-props/normalize.light.min.css";

                link rel="icon mask-icon" href="/favicon.svg";
                link rel="manifest" href="/app.webmanifest";
                title { "Cloud Casino" }

                script defer src="//unpkg.com/alpinejs" {}
                script defer src="//unpkg.com/htmx.org" {}
                script defer src="//unpkg.com/lucide" {}

                meta name="viewport" content="width=device-width, initial-scale=1.0";
                meta charset="utf-8";
                meta name="description" content="Bet (fake) money on the weather!";
                meta name="theme-color" content="#006CCF";

                script defer src="/main.js" {}
            }

            body {
                (body)
            }
        }
    }
}
