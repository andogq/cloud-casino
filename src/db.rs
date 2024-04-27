use sqlx::{Executor, SqlitePool};

pub async fn initialise(pool: &SqlitePool) {
    // Set up the tables in the DB
    pool.execute("CREATE TABLE users (id INTEGER, nonce TEXT, last_request DATETIME);")
        .await
        .unwrap();
}
