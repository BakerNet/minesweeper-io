use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Router,
};
use axum_login::tower_sessions::Session;
use oauth2::CsrfToken;
use serde::Deserialize;

use crate::backend::users::{AuthSession, Credentials};

use super::users::OAuthCreds;

pub const NEXT_URL_KEY: &str = "auth.next-url";
pub const CSRF_STATE_KEY: &str = "oauth.csrf-state";
pub const REDIRECT_URL: &str = "/oauth/callback";
pub const OAUTH_TARGET: &str = "oauth.target";

#[derive(Debug, Clone, Deserialize)]
pub struct AuthzResp {
    code: String,
    state: CsrfToken,
}

pub fn router<T>() -> Router<T>
where
    T: Clone + Send + Sync + 'static,
{
    Router::new().route("/oauth/callback", get(oauth_callback))
}

async fn oauth_callback(
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
