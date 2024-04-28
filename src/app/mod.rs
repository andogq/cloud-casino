mod views;

use axum::{extract::State, routing::get, Router};
use maud::Markup;

use crate::{Ctx, MELBOURNE};

use self::views::bet_form::BetFormValue;

async fn index(State(ctx): State<Ctx>) -> Markup {
    let forecast = ctx.weather_service.get_forecast(MELBOURNE).await;

    views::page(views::shell::render(
        183.40,
        forecast,
        Some(BetFormValue {
            rain: true,
            min_temp: 19.0,
            max_temp: 20.0,
            wager: 61.50,
        }),
        1,
    ))
}

pub fn init() -> Router<Ctx> {
    Router::new().route("/", get(index))
}
