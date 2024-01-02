use cfg_if::cfg_if;

use leptos::*;
use leptos_router::*;
use leptos_use::use_window;
use serde::{Deserialize, Serialize};

use crate::models::user::User;

cfg_if! { if #[cfg(feature="ssr")] {
    use axum_login::tower_sessions::Session;
    use crate::backend::{
        auth::{CSRF_STATE_KEY, NEXT_URL_KEY, OAUTH_TARGET},
        users::AuthSession,
    };
}}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OAuthTarget {
    GOOGLE,
    REDDIT,
    GITHUB,
}

#[server(GetUser, "/api")]
pub async fn get_user() -> Result<Option<User>, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::ServerError("Unable to find auth session".to_string()))?;
    Ok(auth_session.user)
}

#[server(LogIn, "/api")]
pub async fn login(target: OAuthTarget, next: Option<String>) -> Result<String, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::ServerError("Unable to find auth session".to_string()))?;
    let session = use_context::<Session>()
        .ok_or_else(|| ServerFnError::ServerError("Unable to find session".to_string()))?;
    let (auth_url, csrf_state) = auth_session.backend.authorize_url(target);

    log::debug!("{}", auth_url);
    log::debug!("{}", csrf_state.secret());

    session
        .insert(CSRF_STATE_KEY, csrf_state.secret())
        .await
        .expect("Serialization should not fail.");

    session
        .insert(OAUTH_TARGET, target)
        .await
        .expect("Serialization should not fail.");

    session
        .insert(NEXT_URL_KEY, next)
        .await
        .expect("Serialization should not fail.");

    Ok(auth_url.as_str().to_string())
}

#[component]
pub fn Login(
    login: Action<LogIn, Result<String, ServerFnError>>,
    target: OAuthTarget,
) -> impl IntoView {
    let (target_str, target_readable) = match target {
        OAuthTarget::GOOGLE => ("GOOGLE", "Log in with Google"),
        OAuthTarget::REDDIT => ("REDDIT", "Log in with Reddit"),
        OAuthTarget::GITHUB => ("GITHUB", "Log in with Github"),
    };

    create_effect(move |_| {
        if let Some(Ok(url)) = login.value().get() {
            let window = use_window();
            let window = window.as_ref();
            if let Some(window) = window {
                let _ = window.open_with_url_and_target(&url, "_self");
            }
        }
    });

    view! {
        <ActionForm action=login>
            <input type="hidden" name="target" value=target_str />
            <input type="submit" value=target_readable />
        </ActionForm>
    }
}

#[server(LogOut, "/api")]
pub async fn logout() -> Result<(), ServerFnError> {
    let mut auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::ServerError("Unable to find auth session".to_string()))?;

    match auth_session.logout().await {
        Ok(_) => {
            leptos_axum::redirect("/");
            Ok(())
        }
        Err(_) => Err(ServerFnError::ServerError(
            "Problem logging out".to_string(),
        )),
    }
}

#[component]
pub fn LogOut(logout: Action<LogOut, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <ActionForm action=logout>
            <input type="submit" value="Log out"/>
        </ActionForm>
    }
}
