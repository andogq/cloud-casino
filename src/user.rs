use std::collections::HashMap;

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use tower_sessions::Session;

const INITIAL_BALANCE: f64 = 100.0;

#[derive(Clone, Serialize, Deserialize)]
pub struct PayOut {
    date: OffsetDateTime,
    amount: f64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Bet {
    /// Money placed on the bet.
    pub wager: f64,

    /// Choice of rain outcome.
    pub rain: bool,

    /// Choice of average temperature.
    pub temperature: f64,

    /// Confidence interval of temperature.
    pub confidence: f64,

    /// Date the bet was placed.
    pub placed: OffsetDateTime,

    /// Pay out Information
    pub pay_out: Option<PayOut>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserData {
    pub last_request: OffsetDateTime,
    pub balance: f64,

    pub bets: HashMap<Date, Bet>,

    pub outstanding_bets: Vec<Date>,
}

impl Default for UserData {
    fn default() -> Self {
        Self {
            last_request: OffsetDateTime::now_utc(),
            balance: INITIAL_BALANCE,

            bets: HashMap::new(),
            outstanding_bets: Vec::new(),
        }
    }
}

pub struct User {
    pub data: UserData,
    session: Session,
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
        user.data.last_request = OffsetDateTime::now_utc();
        user.update_session().await;

        Ok(user)
    }
}
