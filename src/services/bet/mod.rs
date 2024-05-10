mod db;

use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::user::User;

use self::db::Db;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetOutcome {
    pub rain: bool,
    pub temperature: bool,
    pub payout: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BetRecord {
    pub bet: Bet,

    /// Payout calculated when bet was placed
    pub locked_payout: Payout,

    /// Outcome of the bet
    pub outcome: Option<BetOutcome>,
}

impl BetRecord {
    pub fn outcome(&self, weather: &Weather) -> BetOutcome {
        let rain = self.bet.rain == weather.rain;
        let temperature = (self.bet.temperature - weather.temperature).abs() <= self.bet.range;

        return BetOutcome {
            rain,
            temperature,
            payout: [
                rain.then_some(self.locked_payout.rain),
                temperature.then_some(self.locked_payout.temperature),
            ]
            .into_iter()
            .flatten()
            .sum(),
        };
    }
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
    const RAIN_MULTIPLIER: f64 = 1.25;

    fn day_multiplier(date: NaiveDate) -> f64 {
        Self::DAY_MULTIPLIER * (date - Utc::now().date_naive()).num_days() as f64
    }

    pub fn rain_multiplier(date: NaiveDate) -> f64 {
        Self::day_multiplier(date) + Self::RAIN_MULTIPLIER
    }

    pub fn temperature_multiplier(date: NaiveDate, bet: &Bet, forecast: &Forecast) -> f64 {
        Self::day_multiplier(date)
            + (bet.range / (forecast.maximum_temperature - forecast.minimum_temperature)
                * Self::MAX_TEMPERATURE_MULTIPLIER)
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

    pub async fn place(&self, user: &mut User, date: NaiveDate, bet: Bet, payout: Payout) {
        // Insert the bet into the database
        self.db.upsert_bet(date, &bet, user).await;

        // Update the list of oustanding bets
        if !user.data.outstanding_bets.contains(&date) {
            user.data.outstanding_bets.push(date);
        }

        user.update_session().await;
    }

    /// Find a bet for the given date.
    pub async fn find_bet(&self, date: NaiveDate) -> Option<Bet> {
        self.db.find_bet(date).await
    }

    pub async fn payout(&self, user: &mut User) {
        let now = Utc::now().date_naive();

        let mut ready_bets = vec![];
        for date in std::mem::take(&mut user.data.outstanding_bets) {
            if date < now {
                // Date is ready to be processed
                ready_bets.push(date);
            } else {
                // Not ready to be processed yet, add it back
                user.data.outstanding_bets.push(date);
            }
        }

        for date in ready_bets {
            let weather = self
                .weather_service
                .get_historical_weather(date)
                .await
                .unwrap();

            let bet = user.data.bets.get_mut(&date).unwrap();
            bet.outcome = Some(bet.outcome(&weather));

            user.data.balance += bet.outcome.as_ref().unwrap().payout;
        }

        user.update_session().await;
    }

    pub async fn get_ready(&self, user: &User) -> Vec<(NaiveDate, BetOutcome)> {
        use futures::stream::{FuturesUnordered, StreamExt};
        let now = Utc::now().date_naive();

        user.data
            .outstanding_bets
            .iter()
            .filter(|date| date < &&now)
            .map(|date| async {
                (
                    date.clone(),
                    user.data.bets[date].outcome(
                        &self
                            .weather_service
                            .get_historical_weather(*date)
                            .await
                            .unwrap(),
                    ),
                )
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
    }
}
