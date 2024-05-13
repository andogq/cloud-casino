use reqwest::{
    header::{ACCEPT, AUTHORIZATION, USER_AGENT},
    Client,
};
use sqlx::SqlitePool;
use url::Url;

use super::state::StateService;

mod env {
    pub static OAUTH_CLIENT_ID: &str = "OAUTH_CLIENT_ID";
    pub static OAUTH_CLIENT_SECRET: &str = "OAUTH_CLIENT_SECRET";
    pub static OAUTH_AUTHORIZATION_URL: &str = "OAUTH_AUTHORIZATION_URL";
    pub static OAUTH_ACCESS_TOKEN_URL: &str = "OAUTH_ACCESS_TOKEN_URL";
}

const DEFAULT_BALANCE: f64 = 100.0;

#[derive(Clone)]
pub struct OAuthService {
    pool: SqlitePool,
    client: Client,
    state: StateService,
}

impl OAuthService {
    pub fn new(pool: SqlitePool, client: Client, state: StateService) -> Self {
        Self {
            pool,
            client,
            state,
        }
    }

    pub async fn generate_authorization_url(&self, provider: impl AsRef<str>) -> Option<Url> {
        let identifier = provider.as_ref().to_uppercase();

        // Fetch required environment variables
        let client_id = std::env::var(format!("{}_{identifier}", env::OAUTH_CLIENT_ID)).ok()?;
        let mut authorization_url = {
            let raw =
                std::env::var(format!("{}_{identifier}", env::OAUTH_AUTHORIZATION_URL)).ok()?;
            Url::parse(&raw).ok()?
        };

        // Generate some state value
        let state = self.state.generate(format!("{identifier}-oauth")).await;

        // Build
        authorization_url
            .query_pairs_mut()
            .append_pair("client_id", &client_id)
            .append_pair("state", &state);

        Some(authorization_url)
    }

    pub async fn complete_flow(
        &self,
        provider: impl AsRef<str>,
        state: String,
        code: String,
    ) -> Option<i64> {
        let identifier = provider.as_ref().to_uppercase();

        // Make sure state is valid
        if !self
            .state
            .redeem(format!("{identifier}-oauth"), state)
            .await
        {
            return None;
        }

        // Fetch required environment variables
        let client_id = std::env::var(format!("{}_{identifier}", env::OAUTH_CLIENT_ID)).ok()?;
        let client_secret =
            std::env::var(format!("{}_{identifier}", env::OAUTH_CLIENT_SECRET)).ok()?;
        let mut access_token_url = {
            let raw =
                std::env::var(format!("{}_{identifier}", env::OAUTH_ACCESS_TOKEN_URL)).ok()?;
            Url::parse(&raw).ok()?
        };

        // Build the URL
        access_token_url
            .query_pairs_mut()
            .append_pair("client_id", &client_id)
            .append_pair("client_secret", &client_secret)
            .append_pair("code", &code);

        // Fetch the access token from the response
        let access_token = self
            .client
            .post(access_token_url)
            .header(ACCEPT, "application/json")
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap()["access_token"]
            .as_str()
            .unwrap()
            .to_string();

        match provider.as_ref() {
            "github" => {
                // Fetch the username from github
                let username = self
                    .client
                    .get("https://api.github.com/user")
                    .header(ACCEPT, "application/json")
                    .header(AUTHORIZATION, format!("Bearer {access_token}"))
                    .header(USER_AGENT, "cloud-casino")
                    .send()
                    .await
                    .unwrap()
                    .json::<serde_json::Value>()
                    .await
                    .unwrap()["login"]
                    .as_str()
                    .unwrap()
                    .to_string();

                return Some(
                    // Attempt to fetch or insert into the database
                    sqlx::query_scalar!(
                        "INSERT INTO users (balance, last_login, created, auth_provider, auth_identifier)
                            VALUES (?, DATETIME(), DATETIME(), 'github', ?)
                            ON CONFLICT(auth_provider, auth_identifier)
                                DO UPDATE SET last_login = DATETIME()
                            RETURNING id;",
                        DEFAULT_BALANCE,
                        username
                    )
                        .fetch_one(&self.pool)
                        .await
                        .unwrap()
                );
            }
            _ => unreachable!("provider hasn't been implemented"),
        }
    }
}
