use anyhow::Result;
use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
};
use axum_login::tower_sessions::Session;
use oauth2::{
    basic::BasicClient, AuthUrl, ClientId, ClientSecret, CsrfToken, RedirectUrl, TokenUrl,
};
use serde::Deserialize;
use std::env;

use crate::{
    auth_client::SpecialClient,
    users::{AuthSession, Credentials, OAuthCreds},
    OAuthTarget,
};

pub const NEXT_URL_KEY: &str = "auth.next-url";
pub const CSRF_STATE_KEY: &str = "oauth.csrf-state";
pub const OAUTH_TARGET: &str = "oauth.target";

pub const REDIRECT_URL: &str = "/oauth/callback";

#[derive(Debug, Clone, Deserialize)]
pub struct AuthzResp {
    code: String,
    state: CsrfToken,
}

pub fn oauth_client(target: OAuthTarget) -> Result<SpecialClient> {
    let (id_key, secret_key, auth_url, token_url) = match target {
        OAuthTarget::Google => (
            "GOOGLE_CLIENT_ID",
            "GOOGLE_CLIENT_SECRET",
            "https://accounts.google.com/o/oauth2/v2/auth",
            "https://oauth2.googleapis.com/token",
        ),
        OAuthTarget::Reddit => (
            "REDDIT_CLIENT_ID",
            "REDDIT_CLIENT_SECRET",
            "https://www.reddit.com/api/v1/authorize",
            "https://www.reddit.com/api/v1/access_token",
        ),
        OAuthTarget::Github => (
            "GITHUB_CLIENT_ID",
            "GITHUB_CLIENT_SECRET",
            "https://github.com/login/oauth/authorize",
            "https://github.com/login/oauth/access_token",
        ),
    };
    let client_id = env::var(id_key)
        .map(ClientId::new)
        .unwrap_or_else(|_| panic!("{id_key} should be provided."));
    let client_secret = env::var(secret_key)
        .map(ClientSecret::new)
        .unwrap_or_else(|_| panic!("{secret_key} should be provided."));
    let redirect_host = env::var("REDIRECT_HOST").expect("REDIRECT_HOST should be provided");

    let auth_url = AuthUrl::new(auth_url.to_string())?;
    let token_url = TokenUrl::new(token_url.to_string())?;
    let client = BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(RedirectUrl::new(redirect_host + REDIRECT_URL)?);

    Ok(client)
}

pub async fn oauth_callback(
    mut auth_session: AuthSession,
    session: Session,
    Query(AuthzResp {
        code,
        state: new_state,
    }): Query<AuthzResp>,
) -> impl IntoResponse {
    let Ok(Some(old_state)) = session.get(CSRF_STATE_KEY).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Ok(Some(oauth_target)) = session.get(OAUTH_TARGET).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let creds = OAuthCreds {
        code,
        old_state,
        new_state,
    };

    let creds = Credentials {
        target: oauth_target,
        creds,
    };

    let user = match auth_session.authenticate(creds).await {
        Ok(Some(user)) => user,
        Ok(None) => return Redirect::to("/").into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if auth_session.login(&user).await.is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Ok(Some(next)) = session.remove::<String>(NEXT_URL_KEY).await {
        Redirect::to(&next).into_response()
    } else {
        Redirect::to("/").into_response()
    }
}
