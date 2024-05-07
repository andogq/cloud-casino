use futures::stream::{FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use time::{Date, OffsetDateTime};

use crate::{user::User, MELBOURNE};

use super::weather::{DayWeather, Forecast, WeatherService};

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
    pub fn outcome(&self, weather: &DayWeather) -> BetOutcome {
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

    fn day_multiplier(date: Date) -> f64 {
        Self::DAY_MULTIPLIER * (date - OffsetDateTime::now_utc().date()).whole_days() as f64
    }

    pub fn rain_multiplier(date: Date) -> f64 {
        Self::day_multiplier(date) + Self::RAIN_MULTIPLIER
    }

    pub fn temperature_multiplier(date: Date, bet: &Bet, forecast: &Forecast) -> f64 {
        Self::day_multiplier(date)
            + (bet.range / (forecast.max - forecast.min) * Self::MAX_TEMPERATURE_MULTIPLIER)
    }

    pub fn max_payout(bet: &Bet, date: Date, forecast: &Forecast) -> Self {
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
    pool: SqlitePool,
    weather_service: WeatherService,
}

impl BetService {
    pub(super) fn new(pool: SqlitePool, weather_service: WeatherService) -> Self {
        Self {
            pool,
            weather_service,
        }
    }

    pub async fn place(&self, user: &mut User, date: Date, bet: Bet, payout: Payout) {
        // Find any previous bets on this day
        let previous_wager = sqlx::query!(
            "DELETE FROM bets WHERE user = ? AND date = ? RETURNING wager;",
            1,
            date
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap()
        .map(|record| record.wager)
        .flatten()
        .unwrap_or(0.0);

        // Restore the user's balance from the previous bet
        user.data.balance += previous_wager;

        // Add the new bet into the DB
        sqlx::query(
            "
            INSERT INTO bets (user, date, temperature, range, rain, wager, time_placed)
                VALUES (?, ?, ?, ?, ?, ?, ?);
        ",
        )
        .bind(1)
        .bind(date)
        .bind(bet.temperature)
        .bind(bet.range)
        .bind(bet.rain)
        .bind(bet.wager)
        .bind(OffsetDateTime::now_utc())
        .execute(&self.pool)
        .await
        .unwrap();

        // Take the balance from the user for this bet
        user.data.balance -= bet.wager;

        // Update the list of oustanding bets
        if !user.data.outstanding_bets.contains(&date) {
            user.data.outstanding_bets.push(date);
        }

        user.update_session().await;
    }

    pub async fn payout(&self, user: &mut User) {
        let now = OffsetDateTime::now_utc().date();

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
                .get_historical(MELBOURNE, date)
                .await
                .unwrap();

            let bet = user.data.bets.get_mut(&date).unwrap();
            bet.outcome = Some(bet.outcome(&weather));

            user.data.balance += bet.outcome.as_ref().unwrap().payout;
        }

        user.update_session().await;
    }

    pub async fn get_ready(&self, user: &User) -> Vec<(Date, BetOutcome)> {
        let now = OffsetDateTime::now_utc().date();

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
                            .get_historical(MELBOURNE, *date)
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
