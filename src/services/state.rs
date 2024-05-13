use base64::{engine::general_purpose::URL_SAFE, Engine};
use chrono::Utc;
use rand::{thread_rng, RngCore};
use sqlx::SqlitePool;

static RANDOM_BUFFER_LEN: usize = 32;

#[derive(Clone)]
pub struct StateService {
    pool: SqlitePool,
}

impl StateService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn generate(&self, namespace: impl AsRef<str>) -> String {
        let namespace = namespace.as_ref();

        // Generate the value
        let value = {
            let mut bytes = vec![0u8; RANDOM_BUFFER_LEN];
            thread_rng().fill_bytes(&mut bytes);

            URL_SAFE.encode(bytes)
        };

        let now = Utc::now();

        // Insert into the DB
        sqlx::query!(
            "INSERT INTO states (namespace, value, generated)
                VALUES (?, ?, ?);",
            namespace,
            value,
            now
        )
        .execute(&self.pool)
        .await
        .unwrap();

        value
    }

    pub async fn redeem(&self, namespace: impl AsRef<str>, value: impl AsRef<str>) -> bool {
        let namespace = namespace.as_ref();
        let value = value.as_ref();
        let now = Utc::now();

        sqlx::query!(
            "UPDATE states
                SET redeemed = ?
                WHERE namespace = ? AND value = ? and redeemed IS NULL
                RETURNING value;",
            now,
            namespace,
            value
        )
        .fetch_optional(&self.pool)
        .await
        .unwrap()
        .is_some()
    }
}
