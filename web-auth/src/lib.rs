use serde::{Deserialize, Serialize};

#[cfg(feature = "ssr")]
mod auth;
#[cfg(feature = "ssr")]
pub mod auth_client;
#[cfg(feature = "ssr")]
mod users;

#[cfg(feature = "ssr")]
pub mod models;

#[cfg(feature = "ssr")]
pub use auth::{
    oauth_callback, oauth_client, CSRF_STATE_KEY, NEXT_URL_KEY, OAUTH_TARGET, REDIRECT_URL,
};
#[cfg(feature = "ssr")]
pub use users::{AuthSession, Backend};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OAuthTarget {
    Google,
    Github,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrontendUser {
    pub display_name: Option<String>,
}

impl FrontendUser {
    pub fn display_name_or_anon(display_name: Option<&String>, is_user: bool) -> String {
        if let Some(name) = display_name {
            name.to_owned()
        } else if is_user {
            "Anonymous".to_string()
        } else {
            "Guest".to_string()
        }
    }
}
