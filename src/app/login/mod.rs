use axum::{
    extract::{Query, State},
    response::Redirect,
    routing::get,
    Router,
};
use maud::Markup;
use serde::Deserialize;
use tower_sessions::Session;

use crate::Ctx;

use self::views::login::Provider;

use super::views::page;

mod views;

async fn render_login(State(ctx): State<Ctx>) -> Markup {
    page(views::login::render(&[Provider {
        name: "GitHub".to_string(),
        icon: "github".to_string(),
        url: ctx
            .services
            .oauth
            .generate_authorization_url("github")
            .await
            .unwrap()
            .to_string(),
    }]))
}

#[derive(Deserialize)]
struct OAuthCallbackParams {
    code: String,
    state: String,
}

async fn callback_github(
    State(ctx): State<Ctx>,
    Query(params): Query<OAuthCallbackParams>,
    session: Session,
) -> Redirect {
    let Some(user_id) = ctx
        .services
        .oauth
        .complete_flow("github", params.state, params.code)
        .await
    else {
        // Try again, something went wrong
        return Redirect::temporary("/login");
    };

    // Insert the user ID in the session
    session.insert("user_id", user_id).await.unwrap();
    session.save().await.unwrap();

    // Redirect to the main page
    Redirect::temporary("/")
}

pub fn init() -> Router<Ctx> {
    Router::new()
        .route("/", get(render_login))
        .route("/callback/github", get(callback_github))
        // TODO: Remove
        .route(
            "/whoami",
            get(|session: Session| async move {
                let id = session.get::<i64>("user_id").await.unwrap();
                format!("the user is {id:?}")
            }),
        )
}
