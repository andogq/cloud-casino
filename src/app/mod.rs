mod views;

use axum::{
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
    Router,
};
use maud::Markup;
use time::Date;

use crate::{user::User, Ctx, MELBOURNE};

use self::views::bet_form::BetFormValue;

async fn index(State(ctx): State<Ctx>, user: User) -> Markup {
    let balance = user.data.balance;
    let forecast = ctx.weather_service.get_forecast(MELBOURNE).await;
    let ready_payouts = crate::payout::count_ready(&user);

    views::page(views::shell::render(
        balance,
        forecast,
        Some(BetFormValue {
            rain: true,
            min_temp: 19.0,
            max_temp: 20.0,
            wager: 61.50,
        }),
        ready_payouts,
    ))
}

async fn place_bet(Path(date): Path<Date>) -> Redirect {
    Redirect::to("/app")
}

pub fn init() -> Router<Ctx> {
    Router::new()
        .route("/", get(index))
        .route("/bet/:date", post(place_bet))
}
