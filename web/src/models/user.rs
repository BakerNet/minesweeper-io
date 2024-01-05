#![cfg(feature = "ssr")]

use axum_login::AuthUser;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

#[derive(Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
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

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        self.access_token.as_bytes()
    }
}

impl User {
    pub async fn get_user(db: &SqlitePool, user_id: i64) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as("select * from users where id = ?")
            .bind(user_id)
            .fetch_optional(db)
            .await
    }

    pub async fn add_user(
        db: &SqlitePool,
        username: &str,
        access_token: &str,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as(
            r#"
            insert into users (username, access_token)
            values (?, ?)
            on conflict(username) do update
            set access_token = excluded.access_token
            returning *
            "#,
        )
        .bind(username)
        .bind(access_token)
        .fetch_one(db)
        .await
    }

    pub async fn update_display_name(
        db: &SqlitePool,
        user_id: i64,
        display_name: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update users set display_name = ? where id = ?")
            .bind(display_name)
            .bind(user_id)
            .execute(db)
            .await
            .map(|_| ())
    }
}
