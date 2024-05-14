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
        html {
            head {
                link rel="stylesheet" type="text/css" href="/app.css";
                link rel="stylesheet" type="text/css" href="https://unpkg.com/open-props";
                link rel="stylesheet" type="text/css" href="https://unpkg.com/open-props/normalize.min.css";

                script defer src="//unpkg.com/alpinejs" {}
                script defer src="//unpkg.com/htmx.org" {}
                script defer src="//unpkg.com/lucide" {}

                meta name="viewport" content="width=device-width, initial-scale=1.0";

                script defer src="/main.js" {}
            }

            body {
                (body)
            }
        }
    }
}
