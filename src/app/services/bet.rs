use time::{Date, OffsetDateTime};

use crate::{user::User, weather::Forecast};

#[derive(Debug)]
pub struct Bet {
    /// Minimum temperature guessed
    pub min: f64,

    /// Maximum temperature guessed
    pub max: f64,

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
            + ((bet.max - bet.min) / (forecast.max - forecast.min)
                * Self::MAX_TEMPERATURE_MULTIPLIER);

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
        let previous_bet = user.data.bets.insert(
            date,
            crate::Bet {
                wager: bet.wager,
                rain: bet.rain,
                min: bet.min,
                max: bet.max,
                // TODO: Replace with updated data structure
                forecast_range: 1.0,
                placed: OffsetDateTime::now_utc(),
                // TODO: Replace with updated data structure
                payout: None,
            },
        );

        if let Some(previous_bet) = previous_bet {
            user.data.balance += previous_bet.wager;
        } else {
            user.data.outstanding_bets.push(date);
        }

        user.data.balance -= bet.wager;

        user.update_session().await;
    }
}
