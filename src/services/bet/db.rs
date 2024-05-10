use chrono::{NaiveDate, Utc};
use futures::StreamExt;
use sqlx::SqlitePool;

use crate::user::User;

use super::{Bet, BetOutcome, Payout};

/// Entire bet record, as it appears in the database.
pub struct BetRecord {
    pub date: NaiveDate,

    /// Temperature that was guessed
    pub temperature: f64,

    /// Range in average temperature
    pub range: f64,

    /// Guess if it will rain
    pub rain: bool,

    /// Wager placed on bet
    pub wager: f64,

    /// Payout if rain is correct
    pub rain_payout: f64,

    /// Payout if temperature is correct
    pub temperature_payout: f64,
}

impl BetRecord {
    pub fn new(date: NaiveDate, bet: Bet, payout: Payout) -> Self {
        Self {
            date,
            temperature: bet.temperature,
            range: bet.range,
            rain: bet.rain,
            wager: bet.wager,
            rain_payout: payout.rain,
            temperature_payout: payout.temperature,
        }
    }
}

impl From<&BetRecord> for Bet {
    fn from(bet: &BetRecord) -> Self {
        Self {
            temperature: bet.temperature,
            range: bet.range,
            rain: bet.rain,
            wager: bet.wager,
        }
    }
}

impl From<BetRecord> for Bet {
    fn from(bet: BetRecord) -> Self {
        Bet::from(&bet)
    }
}

impl From<&BetRecord> for Payout {
    fn from(bet: &BetRecord) -> Self {
        Self {
            rain: bet.rain_payout,
            temperature: bet.temperature_payout,
        }
    }
}

impl From<BetRecord> for Payout {
    fn from(bet: BetRecord) -> Self {
        Payout::from(&bet)
    }
}

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert_bet(&self, bet: &BetRecord, user: &mut User) {
        let user_id = 1;
        let now = Utc::now();

        let tx = self.pool.begin().await.unwrap();

        // Get the current wager
        let previous_wager = sqlx::query_scalar!(
            "SELECT wager
                FROM bets
                WHERE user = ? AND date = ?;",
            user_id,
            bet.date
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap()
        .unwrap_or_default();

        // Insert the new bet
        sqlx::query!(
            "INSERT INTO bets (user, date, temperature, range, rain, wager, rain_payout, temperature_payout, time_placed)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT (user, date) DO UPDATE
                    SET temperature = ?, range = ?, rain = ?, wager = ?, rain_payout = ?, temperature_payout = ?, time_placed = ?;",
            user_id,
            bet.date,
            bet.temperature,
            bet.range,
            bet.rain,
            bet.wager,
            bet.rain_payout,
            bet.temperature_payout,
            now,
            bet.temperature,
            bet.range,
            bet.rain,
            bet.wager,
            bet.rain_payout,
            bet.temperature_payout,
            now
        )
        .execute(&self.pool)
        .await
        .unwrap();

        // TODO: Update the user's balance by the difference
        // sqlx::query!("UPDATE users SET balance = balance + ? WHERE id = ?;")
        user.data.balance += previous_wager - bet.wager;
        user.update_session().await;

        // Finalise the transaction
        tx.commit().await.unwrap();
    }

    pub async fn find_bet(&self, date: NaiveDate) -> Option<BetRecord> {
        let user_id = 1;

        sqlx::query_as!(
            BetRecord,
            "SELECT date, temperature, range, rain, wager, rain_payout, temperature_payout
                FROM bets
                WHERE user = ? and date = ?;",
            user_id,
            date,
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap()
    }

    pub async fn record_payout(&self, date: NaiveDate, outcome: &BetOutcome) {
        let user_id = 1;
        let now = Utc::now();

        sqlx::query!(
            "INSERT INTO payouts (bet_user, bet_date, payout_date, rain_correct, temperature_correct)
                VALUES (?, ?, ?, ?, ?);",
            user_id,
            date,
            now,
            outcome.rain,
            outcome.temperature
        )
        .execute(&self.pool)
        .await
        .unwrap();
    }

    /// Retrieve all bets that are ready to be paid out.
    pub async fn ready_bets(&self) -> Vec<BetRecord> {
        let user_id = 1;

        // WARN: This will mix UTC dates with user locale dates
        let now = Utc::now().date_naive();

        // Select all bets that don't have a corresponding payout
        sqlx::query_as!(
            BetRecord,
            "SELECT date, temperature, range, rain, wager, rain_payout, temperature_payout
                FROM bets
                WHERE user = ?
                    AND date < ?
                    AND (
                        SELECT COUNT(*)
                            FROM payouts
                            WHERE payouts.bet_date = bets.date
                    ) = 0;",
            user_id,
            now
        )
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }
}
