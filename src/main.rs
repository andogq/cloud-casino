mod app;
mod services;
mod user;

use std::{env, net::Ipv4Addr, str::FromStr};

use axum::{http::HeaderValue, routing::get, Router};
use reqwest::header::USER_AGENT;
use services::Services;
use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use tower_http::services::ServeDir;
use tower_sessions::{
    cookie::{time, SameSite},
    Expiry, SessionManagerLayer,
};
use tower_sessions_sqlx_store::SqliteStore;

const MELBOURNE: (f64, f64) = (-37.814, 144.9633);

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
        .with_expiry(Expiry::AtDateTime(time::OffsetDateTime::new_utc(
            time::Date::from_calendar_date(2099, time::Month::December, 31).unwrap(),
            time::Time::MIDNIGHT,
        )));

    let reqwest_client = reqwest::Client::builder()
        .default_headers(
            [(
                USER_AGENT,
                HeaderValue::from_str(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION")
                ))
                .unwrap(),
            )]
            .into_iter()
            .collect(),
        )
        .build()
        .unwrap();

    let app = Router::new()
        .merge(app::init())
        .route("/health", get(|| async { "ok" }))
        .fallback_service(ServeDir::new(&static_dir))
        .layer(session_layer)
        .with_state(Ctx {
            db: pool.clone(),
            services: Services::new(pool, reqwest_client),
        });

    let listener = tokio::net::TcpListener::bind((Ipv4Addr::UNSPECIFIED, port))
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
