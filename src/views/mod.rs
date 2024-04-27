use maud::{html, Markup};

pub mod bet_form;
pub mod forecast;

pub fn page(body: Markup) -> Markup {
    html! {
        (maud::DOCTYPE)
        html {
            head {
                link rel="stylesheet" type="text/css" href="/main.css";

                script defer src="//unpkg.com/alpinejs" {}
                script defer src="//unpkg.com/htmx.org" {}
            }

            body { (body) }
        }
    }
}
