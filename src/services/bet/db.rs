use chrono::{NaiveDate, Utc};
use sqlx::SqlitePool;

use crate::user::UserId;

use super::{Bet, BetOutcome, Payout};

/// Entire bet record, as it appears in the database.
#[derive(Debug, Clone)]
pub struct BetRecord {
    /// Date the bet is for
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

    pub async fn upsert_bet(&self, user: UserId, bet: &BetRecord) {
        let mut tx = self.pool.begin().await.unwrap();

        // Get the current wager
        let previous_wager = sqlx::query_scalar!(
            "SELECT wager
                FROM bets
                WHERE user = ? AND date = ?;",
            user,
            bet.date
        )
        .fetch_optional(tx.as_mut())
        .await
        .unwrap()
        .unwrap_or_default();

        // Insert the new bet
        sqlx::query!(
            "INSERT INTO bets (user, date, temperature, range, rain, wager, rain_payout, temperature_payout)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT (user, date) DO UPDATE
                    SET temperature = ?, range = ?, rain = ?, wager = ?, rain_payout = ?, temperature_payout = ?;",
            // Insert values
            user,
            bet.date,
            bet.temperature,
            bet.range,
            bet.rain,
            bet.wager,
            bet.rain_payout,
            bet.temperature_payout,
            // Update values
            bet.temperature,
            bet.range,
            bet.rain,
            bet.wager,
            bet.rain_payout,
            bet.temperature_payout,
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        // Update the user's balance
        let d_balance = previous_wager - bet.wager;
        let balance = sqlx::query_scalar!(
            "UPDATE users SET balance = balance + ? WHERE id = ? RETURNING balance;",
            d_balance,
            user
        )
        .fetch_one(tx.as_mut())
        .await
        .unwrap();

        if balance >= 0.0 {
            // Finalise the transaction
            tx.commit().await.unwrap();
        } else {
            tx.rollback().await.unwrap();
        }
    }

    pub async fn find_bet(&self, user: UserId, date: NaiveDate) -> Option<BetRecord> {
        sqlx::query_as!(
            BetRecord,
            "SELECT date, temperature, range, rain, wager, rain_payout, temperature_payout
                FROM bets
                WHERE user = ? and date = ?;",
            user,
            date,
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap()
    }

    pub async fn record_payout(&self, user: UserId, date: NaiveDate, outcome: &BetOutcome) {
        // Begin a new transaction
        let mut tx = self.pool.begin().await.unwrap();

        // Record the payout
        sqlx::query!(
            "INSERT INTO payouts (user, date, rain_correct, temperature_correct)
                VALUES (?, ?, ?, ?);",
            user,
            date,
            outcome.rain,
            outcome.temperature
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        sqlx::query!(
            "UPDATE users
                -- Perform the actual balance update
                SET balance = users.balance + d.balance
                FROM (
                    -- Build the balance change amount
                    SELECT (
                        IFNULL(
                            -- Select the temperature payout from the bet
                            (SELECT temperature_payout
                                FROM bets
                                -- Only include the temperature payout if it's correct
                                WHERE ?
                                    AND date = ?
                                    AND user = ?),
                            0
                        ) + IFNULL(
                            -- Do the same for the rain payout
                            (SELECT rain_payout
                                FROM bets
                                WHERE ?
                                    AND date = ?
                                    AND user = ?),
                            0
                        )
                    ) AS balance
                ) AS d
                WHERE id = ?;",
            // Temperature payout information
            outcome.temperature,
            date,
            user,
            // Rain payout information
            outcome.rain,
            date,
            user,
            // User to update
            user
        )
        .execute(tx.as_mut())
        .await
        .unwrap();

        tx.commit().await.unwrap();
    }

    /// Retrieve all bets that are ready to be paid out.
    pub async fn ready_bets(&self, user: UserId) -> Vec<BetRecord> {
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
                            WHERE payouts.date = bets.date
                                AND payouts.user = bets.user
                    ) = 0;",
            user,
            now
        )
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }

    pub async fn get_balance(&self, user: UserId) -> f64 {
        sqlx::query_scalar!("SELECT balance FROM users WHERE id = ?;", user)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }
}
