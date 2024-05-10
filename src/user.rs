use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tower_sessions::Session;

const INITIAL_BALANCE: f64 = 100.0;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserData {
    pub last_request: DateTime<Utc>,
    pub balance: f64,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            last_request: Utc::now(),
            balance: INITIAL_BALANCE,
        }
    }
}

pub struct User {
    pub data: UserData,
    pub session: Session,
}

impl User {
    const SESSION_KEY: &'static str = "user.data";

    pub async fn from_session(session: Session) -> Self {
        Self {
            data: session
                .get::<UserData>(Self::SESSION_KEY)
                .await
                .unwrap()
                .unwrap_or_default(),
            session,
        }
    }

    pub async fn update_session(&self) {
        self.session
            .insert(Self::SESSION_KEY, self.data.clone())
            .await
            .unwrap();
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    /// Perform the extraction.
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state).await?;

        let mut user = Self::from_session(session).await;
        user.data.last_request = Utc::now();
        user.update_session().await;

        Ok(user)
    }
}
