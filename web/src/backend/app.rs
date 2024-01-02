use std::env;

use axum::{error_handling::HandleErrorLayer, http::StatusCode, BoxError, Router};
use axum_login::{
    tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer},
    AuthManagerLayerBuilder,
};
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use sqlx::SqlitePool;
use time::Duration;
use tower::ServiceBuilder;
use tower_sessions::SqliteStore;

use crate::{
    auth::OAuthTarget,
    backend::{auth, users},
};

use super::auth::REDIRECT_URL;

pub struct App {
    pub db: SqlitePool,
    pub google_client: BasicClient,
    pub reddit_client: BasicClient,
    pub github_client: BasicClient,
    pub session_store: SqliteStore,
}

fn oauth_client(target: OAuthTarget) -> Result<BasicClient, Box<dyn std::error::Error>> {
    let (id_key, secret_key, auth_url, token_url) = match target {
        OAuthTarget::GOOGLE => (
            "GOOGLE_CLIENT_ID",
            "GOOGLE_CLIENT_SECRET",
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
        ),
        OAuthTarget::REDDIT => (
            "REDDIT_CLIENT_ID",
            "REDDIT_CLIENT_SECRET",
            "https://www.reddit.com/api/v1/authorize",
            "https://www.reddit.com/api/v1/access_token",
        ),
        OAuthTarget::GITHUB => (
            "GITHUB_CLIENT_ID",
            "GITHUB_CLIENT_SECRET",
            "https://github.com/login/oauth/authorize",
            "https://github.com/login/oauth/access_token",
        ),
    };
    let client_id = env::var(id_key)
        .map(ClientId::new)
        .expect(&format!("{} should be provided.", id_key));
    let client_secret = env::var(secret_key)
        .map(ClientSecret::new)
        .expect(&format!("{} should be provided.", secret_key));
    let redirect_host = env::var("REDIRECT_HOST")
        .map(String::from)
        .expect("REDIRECT_HOST should be provided");
    log::debug!(
        "{:?} {} {} {} {}",
        target,
        id_key,
        secret_key,
        auth_url,
        token_url
    );

    let auth_url = AuthUrl::new(auth_url.to_string())?;
    let token_url = TokenUrl::new(token_url.to_string())?;
    let client = BasicClient::new(client_id, Some(client_secret), auth_url, Some(token_url))
        .set_redirect_uri(RedirectUrl::new(redirect_host + REDIRECT_URL)?);

    Ok(client)
}

impl App {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenvy::dotenv()?;

        let google_client = oauth_client(OAuthTarget::GOOGLE)?;
        let reddit_client = oauth_client(OAuthTarget::REDDIT)?;
        let github_client = oauth_client(OAuthTarget::GITHUB)?;

        let db_url = env::var("DATABASE_URL")
            .map(String::from)
            .expect("DATABASE_URL should be provided");

        let db = SqlitePool::connect(&db_url).await?;
        sqlx::migrate!().run(&db).await?;

        let session_store = SqliteStore::new(db.clone());
        session_store
            .migrate()
            .await
            .expect("Migrations for session store should work");

        Ok(Self {
            db,
            google_client,
            reddit_client,
            github_client,
            session_store,
        })
    }

    pub fn extend_router<T>(self, router: Router<T>) -> Router<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        // Session layer.
        // This uses `tower-sessions` to establish a layer that will provide the session
        // as a request extension.
        let session_layer = SessionManagerLayer::new(self.session_store)
            .with_secure(false)
            .with_same_site(SameSite::Lax) // Ensure we send the cookie from the OAuth redirect.
            .with_expiry(Expiry::OnInactivity(Duration::days(1)));

        // Auth service.
        //
        // This combines the session layer with our backend to establish the auth
        // service which will provide the auth session as a request extension.
        let backend = users::Backend::new(
            self.db,
            self.google_client,
            self.reddit_client,
            self.github_client,
        );
        let auth_service = ServiceBuilder::new()
            .layer(HandleErrorLayer::new(|_: BoxError| async {
                StatusCode::BAD_REQUEST
            }))
            .layer(AuthManagerLayerBuilder::new(backend, session_layer).build());

        router.merge(auth::router()).layer(auth_service)
    }
}
