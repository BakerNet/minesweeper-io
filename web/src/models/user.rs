use cfg_if::cfg_if;

use serde::{Deserialize, Serialize};

cfg_if! { if #[cfg(feature="ssr")] {
    use sqlx::FromRow;
    use axum_login::AuthUser;
}}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct User {
    id: i64,
    pub username: String,
    pub display_name: Option<String>,
    pub access_token: String,
}

// Here we've implemented `Debug` manually to avoid accidentally logging the
// access token.
impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("id", &self.id)
            .field("username", &self.username)
            .field("display_name", &self.display_name)
            .field("access_token", &"[redacted]")
            .finish()
    }
}

cfg_if! { if #[cfg(feature="ssr")] {
    impl AuthUser for User {
        type Id = i64;

        fn id(&self) -> Self::Id {
            self.id
        }

        fn session_auth_hash(&self) -> &[u8] {
            self.access_token.as_bytes()
        }
    }
}}
