pub mod services;
mod views;

use axum::{
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
    Form, Router,
};
use maud::Markup;
use time::{Date, OffsetDateTime};

use crate::{user::User, Ctx, MELBOURNE};

use self::{
    services::bet::{Bet, Payout},
    views::bet_form::BetForm,
};

async fn index(State(ctx): State<Ctx>, maybe_date: Option<Path<Date>>, user: User) -> Markup {
    let balance = user.data.balance;
    let forecast = ctx.weather_service.get_forecast(MELBOURNE).await;
    let ready_payouts = crate::payout::count_ready(&user);

    let date = maybe_date.map(|Path(date)| date);

    // TODO: Get this from *somewhere*
    let bet = Bet {
        temperature: 21.0,
        range: 2.0,
        rain: true,
        wager: 61.5,
    };

    let payout = Payout::calculate(
        &bet,
        date.unwrap_or_else(|| OffsetDateTime::now_utc().date()),
        &forecast[0],
    );

    views::page(views::shell::render(
        balance,
        forecast,
        date,
        bet.into(),
        payout.total(),
        ready_payouts,
    ))
}

async fn place_bet(
    State(ctx): State<Ctx>,
    mut user: User,
    Path(date): Path<Date>,
    Form(bet_form): Form<BetForm>,
) -> Redirect {
    // Construct the bet
    let bet = bet_form.into();

    // Determine the forecast for the day
    let forecast = ctx
        .weather_service
        .get_forecast(MELBOURNE)
        .await
        .into_iter()
        .find(|forecast| forecast.date == date)
        .unwrap();
    let payout = Payout::calculate(&bet, date, &forecast);

    ctx.services.bet.place(&mut user, date, bet, payout).await;

    Redirect::to("/app")
}

async fn calculate_payout(
    State(ctx): State<Ctx>,
    Path(date): Path<Date>,
    Form(bet_form): Form<BetForm>,
) -> Markup {
    let forecast = ctx
        .weather_service
        .get_forecast(MELBOURNE)
        .await
        .into_iter()
        .find(|forecast| forecast.date == date)
        .unwrap();
    let payout = Payout::calculate(&bet_form.into(), date, &forecast);

    views::bet_form::render_maximum_payout(date, payout.total())
}

pub fn init() -> Router<Ctx> {
    Router::new()
        .route("/", get(index))
        .route("/:date", get(index))
        .route("/bet/:date", post(place_bet))
        .route("/bet/:date/payout", get(calculate_payout))
}
