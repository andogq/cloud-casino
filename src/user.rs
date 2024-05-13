use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

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
    type Rejection = Response;

    /// Perform the extraction.
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(|e| e.into_response())?;

        Self::from_session(session)
            .await
            .ok_or(Redirect::temporary("/login").into_response())
    }
}
