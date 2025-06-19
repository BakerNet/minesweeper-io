use async_trait::async_trait;
use axum::http::header::{AUTHORIZATION, USER_AGENT};
use axum_login::{AuthnBackend, UserId};
use oauth2::{
    basic::BasicRequestTokenError, reqwest::ClientBuilder, url::Url, AuthorizationCode, CsrfToken,
    HttpClientError, Scope, TokenResponse,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{auth_client::SpecialClient, models::User, OAuthTarget};

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub target: OAuthTarget,
    pub creds: OAuthCreds,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OAuthCreds {
    pub code: String,
    pub old_state: CsrfToken,
    pub new_state: CsrfToken,
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    email: String,
}

#[derive(Debug, Deserialize)]
struct GithubUserInfo {
    login: String,
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error(transparent)]
    Sqlx(sqlx::Error),

    #[error(transparent)]
    Reqwest(reqwest::Error),

    #[error(transparent)]
    OAuth2(BasicRequestTokenError<HttpClientError<reqwest::Error>>),
}

#[derive(Debug, Clone)]
pub struct Backend {
    db: SqlitePool,
    google_client: SpecialClient,
    github_client: SpecialClient,
    http_client: reqwest::Client,
}

impl Backend {
    pub fn new(db: SqlitePool, google_client: SpecialClient, github_client: SpecialClient) -> Self {
        Self {
            db,
            google_client,
            github_client,
            http_client: ClientBuilder::new()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .expect("Should build client"),
        }
    }

    pub fn get_client(&self, target: OAuthTarget) -> &SpecialClient {
        match target {
            OAuthTarget::Google => &self.google_client,
            OAuthTarget::Github => &self.github_client,
        }
    }

    pub fn authorize_url(&self, target: OAuthTarget) -> (Url, CsrfToken) {
        match target {
            OAuthTarget::Google => self
                .google_client
                .authorize_url(CsrfToken::new_random)
                .add_scope(Scope::new(
                    "https://www.googleapis.com/auth/userinfo.profile".to_string(),
                ))
                .add_scope(Scope::new(
                    "https://www.googleapis.com/auth/userinfo.email".to_string(),
                ))
                .url(),
            OAuthTarget::Github => self
                .github_client
                .authorize_url(CsrfToken::new_random)
                .add_extra_param("duration", "permanent")
                .add_scope(Scope::new("user".to_string()))
                .url(),
        }
    }

    pub async fn update_user_display_name(
        &self,
        user_id: i64,
        display_name: &str,
    ) -> Result<(), BackendError> {
        User::update_display_name(&self.db, user_id, display_name)
            .await
            .map_err(BackendError::Sqlx)
    }
}

#[async_trait]
impl AuthnBackend for Backend {
    type User = User;
    type Credentials = Credentials;
    type Error = BackendError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        // Ensure the CSRF state has not been tampered with.
        if creds.creds.old_state.secret() != creds.creds.new_state.secret() {
            return Ok(None);
        };

        let (token_res, username) = match creds.target {
            OAuthTarget::Google => {
                // Process authorization code, expecting a token response back.
                let token_res = self
                    .google_client
                    .exchange_code(AuthorizationCode::new(creds.creds.code))
                    .request_async(&self.http_client)
                    .await
                    .map_err(Self::Error::OAuth2)?;

                // Use access token to request user info.
                let user_info = self
                    .http_client
                    .get("https://www.googleapis.com/oauth2/v1/userinfo")
                    .header(USER_AGENT.as_str(), "minesweeper-io")
                    .header(
                        AUTHORIZATION.as_str(),
                        format!("Bearer {}", token_res.access_token().secret()),
                    )
                    .send()
                    .await
                    .map_err(Self::Error::Reqwest)?
                    .json::<GoogleUserInfo>()
                    .await
                    .map_err(Self::Error::Reqwest)?;
                (token_res, format!("GOOGLE:{}", user_info.email))
            }
            OAuthTarget::Github => {
                // Process authorization code, expecting a token response back.
                let token_res = self
                    .github_client
                    .exchange_code(AuthorizationCode::new(creds.creds.code))
                    .request_async(&self.http_client)
                    .await
                    .map_err(Self::Error::OAuth2)?;

                // Use access token to request user info.
                let user_info = self
                    .http_client
                    .get("https://api.github.com/user")
                    .header(USER_AGENT.as_str(), "minesweeper-io")
                    .header(
                        AUTHORIZATION.as_str(),
                        format!("Bearer {}", token_res.access_token().secret()),
                    )
                    .send()
                    .await
                    .map_err(Self::Error::Reqwest)?
                    .json::<GithubUserInfo>()
                    .await
                    .map_err(Self::Error::Reqwest)?;
                (token_res, format!("GITHUB:{}", user_info.login))
            }
        };

        // Persist user in our database so we can use `get_user`.
        let user = User::add_user(&self.db, &username, token_res.access_token().secret())
            .await
            .map_err(Self::Error::Sqlx)?;

        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        Ok(User::get_user(&self.db, *user_id)
            .await
            .map_err(Self::Error::Sqlx)?)
    }
}

// We use a type alias for convenience.
//
// Note that we've supplied our concrete backend here.
pub type AuthSession = axum_login::AuthSession<Backend>;
