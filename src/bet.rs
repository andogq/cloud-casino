use time::{Date, OffsetDateTime};

use crate::{
    user::{Bet, User},
    Ctx, MELBOURNE,
};

pub async fn place_bet(
    ctx: Ctx,
    mut user: User,
    date: Date,
    min_temp: f64,
    max_temp: f64,
    rain: bool,
    wager: f64,
) {
    // Get the forecast for the specific day
    let forecast = ctx
        .weather_service
        .get_forecast(MELBOURNE)
        .await
        .into_iter()
        .find(|forecast| forecast.date == date)
        .unwrap();

    // Place a bet on the user
    let bet = Bet {
        wager,
        rain,
        min: min_temp,
        max: max_temp,
        forecast_range: forecast.max - forecast.min,
        placed: OffsetDateTime::now_utc(),
        payout: None,
    };

    let previous_bet = user.data.bets.insert(date, bet);

    if let Some(previous_bet) = previous_bet {
        // Restore user's previous balance
        user.data.balance += previous_bet.wager;
    } else {
        // We need to indicate that this is an outstanding bet
        user.data.outstanding_bets.push(date);
    }

    // Take the money from the user
    user.data.balance -= wager;

    user.update_session().await;
}
