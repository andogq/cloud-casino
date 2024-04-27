use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use tower_sessions::Session;

const INITIAL_BALANCE: f64 = 100.0;

#[derive(Clone, Serialize, Deserialize)]
pub struct User {
    pub last_request: OffsetDateTime,
    pub balance: f64,
}

impl User {
    const SESSION_KEY: &'static str = "user.data";

    pub async fn from_session(session: &Session) -> Option<Self> {
        session.get::<Self>(Self::SESSION_KEY).await.unwrap()
    }

    pub async fn update_session(&self, session: &Session) {
        session
            .insert(Self::SESSION_KEY, self.clone())
            .await
            .unwrap();
    }
}

impl Default for User {
    fn default() -> Self {
        Self {
            last_request: OffsetDateTime::now_utc(),
            balance: INITIAL_BALANCE,
        }
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

        let mut user = Self::from_session(&session).await.unwrap_or_default();
        user.last_request = OffsetDateTime::now_utc();
        user.update_session(&session).await;

        Ok(user)
    }
}
