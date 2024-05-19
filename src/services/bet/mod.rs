mod db;

use chrono::{NaiveDate, Utc};
use chrono_tz::Australia::Melbourne;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::user::UserId;

use self::db::{BetRecord, Db};

use super::weather::{Forecast, Weather, WeatherService};

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

impl BetRecord {
    pub fn outcome(&self, weather: &Weather) -> BetOutcome {
        let rain = self.rain == weather.rain;
        let temperature = (self.temperature - weather.temperature).abs() <= self.range;

        return BetOutcome {
            rain,
            temperature,
            payout: [
                rain.then_some(self.rain_payout),
                temperature.then_some(self.temperature_payout),
            ]
            .into_iter()
            .flatten()
            .sum(),
        };
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetOutcome {
    pub rain: bool,
    pub temperature: bool,
    pub payout: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payout {
    /// Payout if rain is correct
    pub rain: f64,

    /// Payout if temperature is correct
    pub temperature: f64,
}

impl Payout {
    const MAX_TEMPERATURE_MULTIPLIER: f64 = 5.0;
    const DAY_MULTIPLIER: f64 = 0.2;
    const RAIN_MULTIPLIER: f64 = 0.75;

    fn day_multiplier(date: NaiveDate) -> f64 {
        // TODO: This must be locale aware, currently is mixing UTC date with local date
        Self::DAY_MULTIPLIER * (date - Utc::now().date_naive()).num_days() as f64
    }

    pub fn rain_multiplier(date: NaiveDate) -> f64 {
        Self::day_multiplier(date) + Self::RAIN_MULTIPLIER
    }

    pub fn temperature_multiplier(date: NaiveDate, bet: &Bet, forecast: &Forecast) -> f64 {
        let x = 1.0 - (bet.range / (forecast.maximum_temperature - forecast.minimum_temperature));
        let y = x.clamp(0.0, 1.0).powf(3.0);

        Self::day_multiplier(date) + (y * Self::MAX_TEMPERATURE_MULTIPLIER)
    }

    pub fn max_payout(bet: &Bet, date: NaiveDate, forecast: &Forecast) -> Self {
        Self {
            rain: Self::rain_multiplier(date) * bet.wager,
            temperature: Self::temperature_multiplier(date, &bet, forecast) * bet.wager,
        }
    }

    pub fn total(&self) -> f64 {
        self.rain + self.temperature
    }
}

#[derive(Clone, Debug, thiserror::Error)]
pub enum BetError {
    #[error("cannot create a bet for today")]
    Today,

    #[error("cannot create a bet that's less than $0")]
    NegativeBet,
}

#[derive(Clone)]
pub struct BetService {
    weather_service: WeatherService,
    db: Db,
}

impl BetService {
    pub(super) fn new(pool: SqlitePool, weather_service: WeatherService) -> Self {
        Self {
            weather_service,
            db: Db::new(pool),
        }
    }

    /// Place a bet for the given user and date with the specified payout.
    pub async fn place(
        &self,
        user: UserId,
        date: NaiveDate,
        bet: Bet,
        payout: Payout,
    ) -> Result<(), BetError> {
        // Can't place bets that are less than zero
        if bet.wager < 0.0 {
            return Err(BetError::NegativeBet);
        }

        // Determine today's date
        let today = Utc::now().with_timezone(&Melbourne).naive_local().date();

        // Make sure not betting on today, and that the bet amount isn't below zero
        if date == today {
            return Err(BetError::Today);
        }

        // Insert the bet into the database
        self.db
            .upsert_bet(user, &BetRecord::new(date, bet, payout))
            .await;

        Ok(())
    }

    /// Find a bet for the given date.
    pub async fn find_bet(&self, user: UserId, date: NaiveDate) -> Option<Bet> {
        self.db.find_bet(user, date).await.map(|bet| bet.into())
    }

    // Payout all ready bets for the user
    pub async fn payout(&self, user: UserId) {
        let ready_bets = self.db.ready_bets(user).await;

        for bet in ready_bets {
            // Determine the outcome
            let outcome = bet.outcome(
                &self
                    .weather_service
                    .get_historical_weather(bet.date)
                    .await
                    .unwrap(),
            );

            // Mark this bet as payed out
            self.db.record_payout(user, bet.date, &outcome).await;
        }
    }

    pub async fn get_ready(&self, user: UserId) -> Vec<(NaiveDate, BetOutcome)> {
        use futures::stream::FuturesUnordered;

        self.db
            .ready_bets(user)
            .await
            .into_iter()
            .map(|bet| async move {
                Some((
                    bet.date,
                    bet.outcome(
                        &self
                            .weather_service
                            .get_historical_weather(bet.date)
                            .await?,
                    ),
                ))
            })
            .collect::<FuturesUnordered<_>>()
            .filter_map(|weather| async { weather })
            .collect::<Vec<_>>()
            .await
    }

    pub async fn get_balance(&self, user: UserId) -> f64 {
        self.db.get_balance(user).await
    }
}
