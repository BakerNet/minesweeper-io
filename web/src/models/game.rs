use cfg_if::cfg_if;

use minesweeper::cell::PlayerCell;
use serde::{Deserialize, Serialize};

use super::user::User;

cfg_if! { if #[cfg(feature="ssr")] {
    use sqlx::{FromRow, SqlitePool, types::Json};
}}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Game {
    pub game_id: String,
    pub owner: i64, // User.id
    pub rows: i64,
    pub cols: i64,
    pub num_mines: i64,
    pub max_players: u8,
    pub is_completed: bool,
    pub is_started: bool,
    #[sqlx(json)]
    pub final_board: Vec<Vec<PlayerCell>>,
}

#[cfg(feature = "ssr")]
impl Game {
    pub async fn get_game(db: &SqlitePool, game_id: &str) -> Result<Option<Game>, sqlx::Error> {
        sqlx::query_as("select * from games where game_id = ?")
            .bind(game_id)
            .fetch_optional(db)
            .await
    }

    pub async fn create_game(
        db: &SqlitePool,
        game_id: &str,
        owner: &User,
        rows: i64,
        cols: i64,
        num_mines: i64,
        max_players: u8,
    ) -> Result<Game, sqlx::Error> {
        sqlx::query_as(
            r#"
            insert into games (game_id, owner, rows, cols, num_mines, max_players)
            values (?, ?, ?, ?, ?, ?)
            returning *
            "#,
        )
        .bind(game_id)
        .bind(owner.id)
        .bind(rows)
        .bind(cols)
        .bind(num_mines)
        .bind(max_players)
        .fetch_one(db)
        .await
    }

    pub async fn start_game(db: &SqlitePool, game_id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("update users set is_started = 1 where id = ?")
            .bind(game_id)
            .execute(db)
            .await
            .map(|_| ())
    }

    pub async fn complete_game(
        db: &SqlitePool,
        game_id: i64,
        final_board: Vec<Vec<PlayerCell>>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update users set is_completed = 1, final_board = ? where id = ?")
            .bind(Json(final_board))
            .bind(game_id)
            .execute(db)
            .await
            .map(|_| ())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Player {
    pub game_id: String,
    pub user: i64, // User.id
    pub player: u8,
    pub dead: bool,
    pub score: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct PlayerUser {
    pub game_id: String,
    pub user: i64, // User.id
    pub username: String,
    pub dead: bool,
    pub score: i64,
    pub display_name: Option<String>,
    pub player: u8,
}

impl PlayerUser {
    pub fn to_player(&self) -> Player {
        Player {
            game_id: self.game_id.clone(),
            user: self.user,
            player: self.player,
            dead: self.dead,
            score: self.score,
        }
    }
}

#[cfg(feature = "ssr")]
impl Player {
    pub async fn get_players(
        db: &SqlitePool,
        game_id: &str,
    ) -> Result<Vec<PlayerUser>, sqlx::Error> {
        sqlx::query_as(
            "select players.*, users.username, users.display_name from players inner join users on players.user = users.id where players.game_id = ? ",
        )
        .bind(game_id)
        .fetch_all(db)
        .await
    }

    pub async fn add_player(
        db: &SqlitePool,
        game_id: &str,
        user: &User,
        player: u8,
    ) -> Result<(), sqlx::Error> {
        sqlx::query_as(
            r#"
            insert into players (game_id, user, player)
            values (?, ?, ?)
            "#,
        )
        .bind(game_id)
        .bind(user.id)
        .bind(player)
        .fetch_one(db)
        .await
    }

    pub async fn set_score(
        db: &SqlitePool,
        game_id: &str,
        player: u8,
        score: i64,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update players set score = ? where game_id = ? and player = ?")
            .bind(score)
            .bind(game_id)
            .bind(player)
            .execute(db)
            .await
            .map(|_| ())
    }

    pub async fn set_dead(
        db: &SqlitePool,
        game_id: &str,
        player: u8,
        dead: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update players set dead = ? where game_id = ? and player = ?")
            .bind(dead)
            .bind(game_id)
            .bind(player)
            .execute(db)
            .await
            .map(|_| ())
    }
}
