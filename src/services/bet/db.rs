use chrono::{NaiveDate, Utc};
use sqlx::SqlitePool;

use crate::user::User;

use super::Bet;

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn upsert_bet(&self, date: NaiveDate, bet: &Bet, user: &mut User) {
        let user_id = 1;
        let now = Utc::now();

        let tx = self.pool.begin().await.unwrap();

        // Get the current wager
        let previous_wager = sqlx::query_scalar!(
            "SELECT wager
                FROM bets
                WHERE user = ? AND date = ?;",
            user_id,
            date
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap()
        .unwrap_or_default();

        // Insert the new bet
        sqlx::query!(
            "INSERT INTO bets (user, date, temperature, range, rain, wager, time_placed)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT (user, date) DO UPDATE
                    SET temperature = ?, range = ?, rain = ?, wager = ?, time_placed = ?;",
            user_id,
            date,
            bet.temperature,
            bet.range,
            bet.rain,
            bet.wager,
            now,
            bet.temperature,
            bet.range,
            bet.rain,
            bet.wager,
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

    pub async fn find_bet(&self, date: NaiveDate) -> Option<Bet> {
        let user_id = 1;

        sqlx::query_as!(
            Bet,
            "SELECT temperature, range, rain, wager
                FROM bets
                WHERE user = ? and date = ?;",
            user_id,
            date,
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap()
    }
}
