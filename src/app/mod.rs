mod views;

use axum::{routing::get, Router};
use maud::Markup;

use crate::Ctx;

async fn index() -> Markup {
    views::page(views::shell::render())
}

pub fn init() -> Router<Ctx> {
    Router::new().route("/", get(index))
}
