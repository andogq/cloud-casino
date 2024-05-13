use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tower_sessions::Session;

#[derive(Clone, Serialize, Deserialize)]
pub struct Payout {
    pub date: OffsetDateTime,
    pub amount: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Bet {
    /// Money placed on the bet.
    pub wager: f64,

    /// Choice of rain outcome.
    pub rain: bool,

    /// Minimum of temperature range.
    pub min: f64,

    /// Maximum of temperature range.
    pub max: f64,

    /// The temperature range of the forecast at time of bet.
    pub forecast_range: f64,

    /// Date the bet was placed.
    pub placed: OffsetDateTime,

    /// Payout Information
    pub payout: Option<Payout>,
}

#[derive(sqlx::Type, Clone, Copy, Debug)]
#[sqlx(transparent)]
pub struct UserId(i64);

impl UserId {
    const SESSION_KEY: &'static str = "user_id";

    pub async fn from_session(session: Session) -> Option<Self> {
        Some(Self(session.get::<i64>(Self::SESSION_KEY).await.unwrap()?))
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for UserId
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    /// Perform the extraction.
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;

        Self::from_session(session)
            .await
            .ok_or((StatusCode::UNAUTHORIZED, "unauthorised"))
    }
}
