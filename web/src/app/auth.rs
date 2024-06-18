use leptos::*;
use leptos_router::*;
use leptos_use::use_window;
use serde::{Deserialize, Serialize};

use crate::components::button_class;
#[cfg(feature = "ssr")]
use crate::{
    backend::{AuthSession, CSRF_STATE_KEY, NEXT_URL_KEY, OAUTH_TARGET},
    models::user::User,
};
#[cfg(feature = "ssr")]
use axum_login::tower_sessions::Session;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrontendUser {
    pub display_name: Option<String>,
}

impl FrontendUser {
    pub fn display_name_or_anon(display_name: &Option<String>, is_user: bool) -> String {
        if let Some(name) = display_name {
            name.to_owned()
        } else if is_user {
            "Anonymous".to_string()
        } else {
            "Guest".to_string()
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OAuthTarget {
    Google,
    Reddit,
    Github,
}

#[cfg(feature = "ssr")]
pub async fn get_user() -> Result<Option<User>, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    Ok(auth_session.user)
}

#[server(GetUser, "/api")]
pub async fn get_frontend_user() -> Result<Option<FrontendUser>, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    Ok(auth_session.user.map(|u| FrontendUser {
        display_name: u.display_name,
    }))
}

#[server(LogIn, "/api")]
pub async fn login(target: OAuthTarget, next: Option<String>) -> Result<String, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    let session = use_context::<Session>()
        .ok_or_else(|| ServerFnError::new("Unable to find session".to_string()))?;
    let (auth_url, csrf_state) = auth_session.backend.authorize_url(target);

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
    let (target_str, target_readable, target_colors) = match target {
        OAuthTarget::Google => (
            "GOOGLE",
            "Log in with Google",
            "bg-blue-400 text-white hover:bg-blue-600",
        ),
        OAuthTarget::Reddit => (
            "REDDIT",
            "Log in with Reddit",
            "bg-orange-600 text-white hover:bg-orange-800",
        ),
        OAuthTarget::Github => (
            "GITHUB",
            "Log in with Github",
            "bg-zinc-800 text-white hover:bg-zinc-900",
        ),
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
        <ActionForm action=login class="w-full max-w-xs h-8">
            <input type="hidden" name="target" value=target_str/>
            <button
                type="submit"
                class=button_class(Some("w-full max-w-xs h-8"), Some(target_colors))
                disabled=login.pending()
            >
                {target_readable}
            </button>
        </ActionForm>
    }
}

#[server(LogOut, "/api")]
pub async fn logout() -> Result<(), ServerFnError> {
    let mut auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;

    match auth_session.logout().await {
        Ok(_) => {
            leptos_axum::redirect("/");
            Ok(())
        }
        Err(_) => Err(ServerFnError::new("Problem logging out".to_string())),
    }
}

#[component]
pub fn LogOut(logout: Action<LogOut, Result<(), ServerFnError>>) -> impl IntoView {
    view! {
        <ActionForm action=logout class="w-full max-w-xs h-12">
            <button
                type="submit"
                class=button_class(
                    Some("w-full max-w-xs h-12"),
                    Some("bg-red-400 text-black hover:bg-red-500/90"),
                )

                disabled=logout.pending()
            >
                "Log out"
            </button>
        </ActionForm>
    }
}
