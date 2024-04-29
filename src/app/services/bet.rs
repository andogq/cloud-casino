use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};

use crate::{user::User, weather::Forecast};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bet {
    /// Temperature that was guessed
    pub temperature: f64,

    /// Range in average temperature
    pub range: f64,

    /// Guess if it will rain
    pub rain: bool,

    /// Wager placed on bet
    pub wager: f64,
}

#[derive(Debug, Clone)]
pub struct Payout {
    /// Payout if rain is correct
    pub rain: f64,

    /// Payout if temperature is correct
    pub temperature: f64,
}

impl Payout {
    const MAX_TEMPERATURE_MULTIPLIER: f64 = 5.0;
    const DAY_MULTIPLIER: f64 = 0.2;
    const RAIN_MULTIPLIER: f64 = 1.25;

    pub fn calculate(bet: &Bet, date: Date, forecast: &Forecast) -> Self {
        let day_multiplier =
            Self::DAY_MULTIPLIER * (date - OffsetDateTime::now_utc().date()).whole_days() as f64;

        let rain_multiplier = day_multiplier + Self::RAIN_MULTIPLIER;
        let temperature_multiplier = day_multiplier
            + (bet.range / (forecast.max - forecast.min) * Self::MAX_TEMPERATURE_MULTIPLIER);

        Self {
            rain: rain_multiplier * bet.wager,
            temperature: temperature_multiplier * bet.wager,
        }
    }

    pub fn total(&self) -> f64 {
        self.rain + self.temperature
    }
}

#[derive(Clone)]
pub struct BetService;

impl BetService {
    pub(super) fn new() -> Self {
        Self
    }

    pub async fn place(&self, user: &mut User, date: Date, bet: Bet, payout: Payout) {
        let wager = bet.wager;

        let previous_bet = user.data.new_bets.insert(date, bet);

        if let Some(previous_bet) = previous_bet {
            user.data.balance += previous_bet.wager;
        } else {
            user.data.outstanding_bets.push(date);
        }

        user.data.balance -= wager;

        user.update_session().await;
    }
}
