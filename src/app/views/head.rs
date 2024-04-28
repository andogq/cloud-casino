use maud::{html, Markup};

pub fn render(balance: f64) -> Markup {
    html! {
        #head {
            h1 #balance { (format!("${balance:.2}")) }
        }
    }
}
