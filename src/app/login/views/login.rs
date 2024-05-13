use maud::{html, Markup};

pub struct Provider {
    pub icon: String,
    pub name: String,
    pub url: String,
}

pub fn render(providers: &[Provider]) -> Markup {
    html! {
        h1 { "Login" }

        p { "Select a provider to authenticate with:" }

        @for provider in providers {
            a .button href=(provider.url) {
                i data-lucide=(provider.icon){}

                span { (provider.name) }
            }
        }
    }
}
