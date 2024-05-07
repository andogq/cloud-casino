mod app;
mod services;
mod user;

use std::{env, net::Ipv4Addr, str::FromStr};

use axum::{routing::get, Router};
use services::{weather::Point, Services};
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use time::macros::datetime;
use tower_http::services::ServeDir;
use tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::SqliteStore;

const MELBOURNE: Point = Point {
    latitude: -37.814,
    longitude: 144.9633,
};

#[derive(Clone)]
pub struct Ctx {
    pub db: SqlitePool,
    pub services: Services,
}

#[tokio::main]
async fn main() {
    let connection_string = env::var("DATABASE_URL")
        .expect("`DATABASE_URL` environment variable must contain a connection string");
    let port = env::var("PORT")
        .expect("`PORT` environment variable must contain a valid port")
        .parse::<u16>()
        .expect("provided port must be between 1 and 65535");
    let static_dir = env::var("STATIC_DIR")
        .expect("`STATIC_DIR` environment variable must be path to static directory");

    println!("starting server on port {port}, serving files from {static_dir}, db at {connection_string}");

    // DB
    let pool = SqlitePool::connect_with(
        SqliteConnectOptions::from_str(&connection_string)
            .unwrap()
            .create_if_missing(true),
    )
    .await
    .unwrap();

    // Run migrations
    sqlx::migrate!().run(&pool).await.unwrap();

    // Set up sessions
    let session_store = SqliteStore::new(pool.clone());
    session_store.migrate().await.unwrap();

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true)
        .with_same_site(SameSite::Lax)
        .with_http_only(true)
        .with_expiry(Expiry::AtDateTime(datetime!(2099 - 01 - 01 0:00 UTC)));

    let app = Router::new()
        .merge(app::init())
        .route("/health", get(|| async { "ok" }))
        .fallback_service(ServeDir::new(&static_dir))
        .layer(session_layer)
        .with_state(Ctx {
            db: pool.clone(),
            services: Services::new(pool),
        });

    let listener = tokio::net::TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
