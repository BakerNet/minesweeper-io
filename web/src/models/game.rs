#![cfg(feature = "ssr")]
use minesweeper_lib::{cell::PlayerCell, client::ClientPlayer};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, SqlitePool};

use super::user::User;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
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
    pub final_board: Option<Vec<Vec<PlayerCell>>>, // todo - remove final from name
}

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
            insert into games (game_id, owner, rows, cols, num_mines, max_players, final_board)
            values (?, ?, ?, ?, ?, ?, ?)
            returning *
            "#,
        )
        .bind(game_id)
        .bind(owner.id)
        .bind(rows)
        .bind(cols)
        .bind(num_mines)
        .bind(max_players)
        .bind(Json(None::<Vec<Vec<PlayerCell>>>))
        .fetch_one(db)
        .await
    }

    pub async fn start_game(db: &SqlitePool, game_id: &str) -> Result<(), sqlx::Error> {
        sqlx::query("update games set is_started = 1 where game_id = ?")
            .bind(game_id)
            .execute(db)
            .await
            .map(|_| ())
    }

    pub async fn save_board(
        db: &SqlitePool,
        game_id: &str,
        board: Vec<Vec<PlayerCell>>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update games set final_board = ? where game_id = ?")
            .bind(Json(board))
            .bind(game_id)
            .execute(db)
            .await
            .map(|_| ())
    }

    pub async fn complete_game(
        db: &SqlitePool,
        game_id: &str,
        final_board: Vec<Vec<PlayerCell>>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update games set is_completed = 1, final_board = ? where game_id = ?")
            .bind(Json(final_board))
            .bind(game_id)
            .execute(db)
            .await
            .map(|_| ())
    }

    pub async fn set_all_games_completed(db: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("update games set is_completed = 1")
            .execute(db)
            .await
            .map(|_| ())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Player {
    pub game_id: String,
    pub user: i64, // User.id
    pub player: u8,
    pub dead: bool,
    pub score: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
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
        sqlx::query(
            r#"
            insert into players (game_id, user, player)
            values (?, ?, ?)
            "#,
        )
        .bind(game_id)
        .bind(user.id)
        .bind(player)
        .execute(db)
        .await
        .map(|_| ())
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

    pub async fn update_players(
        db: &SqlitePool,
        game_id: &str,
        players: Vec<ClientPlayer>,
    ) -> Result<(), sqlx::Error> {
        let mut transaction = db.begin().await?;
        for p in players {
            sqlx::query("update players set dead = ?, score = ? where game_id = ? and player = ?")
                .bind(p.dead)
                .bind(p.score as i64)
                .bind(game_id)
                .bind(p.player_id as u8)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}
