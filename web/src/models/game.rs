#![cfg(feature = "ssr")]
use chrono::{DateTime, Utc};
use minesweeper_lib::{
    cell::PlayerCell,
    client::ClientPlayer,
    game::{Play, PlayOutcome},
};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, SqlitePool};

use super::user::User;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Game {
    pub game_id: String,
    pub owner: Option<i64>, // User.id
    pub rows: i64,
    pub cols: i64,
    pub num_mines: i64,
    pub max_players: u8,
    pub is_completed: bool,
    pub is_started: bool,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    #[sqlx(json)]
    pub final_board: Option<Vec<Vec<PlayerCell>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct SimpleGameWithPlayers {
    pub game_id: String,
    pub owner: Option<i64>, // User.id
    pub rows: i64,
    pub cols: i64,
    pub num_mines: i64,
    pub max_players: u8,
    pub is_completed: bool,
    pub is_started: bool,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub num_players: u8,
}

pub struct GameParameters {
    pub rows: i64,
    pub cols: i64,
    pub num_mines: i64,
    pub max_players: u8,
}

impl Game {
    pub async fn get_game(db: &SqlitePool, game_id: &str) -> Result<Option<Game>, sqlx::Error> {
        sqlx::query_as("select * from games where game_id = ?")
            .bind(game_id)
            .fetch_optional(db)
            .await
    }

    pub async fn get_games_with_players<T>(
        db: &SqlitePool,
        game_ids: &[T],
    ) -> Result<Vec<SimpleGameWithPlayers>, sqlx::Error>
    where
        T: AsRef<str> + Send + Sync,
    {
        let params = format!("?{}", ", ?".repeat(game_ids.len() - 1));
        let query_str = format!("select game_id, owner, rows, cols, num_mines, max_players, is_completed, is_started, start_time, end_time, ( select count(*) from players where players.game_id = games.game_id ) as num_players from games where game_id in ( {} )", params);

        let mut query = sqlx::query_as(&query_str);
        for i in game_ids {
            query = query.bind(i.as_ref());
        }
        query.fetch_all(db).await
    }

    pub async fn create_game(
        db: &SqlitePool,
        game_id: &str,
        owner: &Option<User>,
        game_parameters: GameParameters,
    ) -> Result<Game, sqlx::Error> {
        let id = owner.as_ref().map(|u| u.id);
        sqlx::query_as(
            r#"
            insert into games (game_id, owner, rows, cols, num_mines, max_players, final_board)
            values (?, ?, ?, ?, ?, ?, ?)
            returning *
            "#,
        )
        .bind(game_id)
        .bind(id)
        .bind(game_parameters.rows)
        .bind(game_parameters.cols)
        .bind(game_parameters.num_mines)
        .bind(game_parameters.max_players)
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

    pub async fn set_start_time(
        db: &SqlitePool,
        game_id: &str,
        timestamp: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("update games set start_time = ? where game_id = ?")
            .bind(timestamp)
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
        end_time: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "update games set is_completed = 1, final_board = ?, end_time = ? where game_id = ?",
        )
        .bind(Json(final_board))
        .bind(end_time)
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
    pub user: Option<i64>, // User.id
    pub nickname: Option<String>,
    pub player: u8,
    pub dead: bool,
    pub victory_click: bool,
    pub top_score: bool,
    pub score: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct PlayerUser {
    pub game_id: String,
    pub user: Option<i64>, // User.id
    pub nickname: Option<String>,
    pub dead: bool,
    pub victory_click: bool,
    pub top_score: bool,
    pub score: i64,
    pub display_name: Option<String>,
    pub player: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct PlayerGame {
    pub game_id: String,
    pub player: u8,
    pub dead: bool,
    pub victory_click: bool,
    pub top_score: bool,
    pub score: i64,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub rows: i64,
    pub cols: i64,
    pub num_mines: i64,
    pub max_players: u8,
}

impl Player {
    pub async fn get_players(
        db: &SqlitePool,
        game_id: &str,
    ) -> Result<Vec<PlayerUser>, sqlx::Error> {
        sqlx::query_as(
            "select players.*, users.display_name from players left join users on players.user = users.id where players.game_id = ?",
        )
        .bind(game_id)
        .fetch_all(db)
        .await
    }

    pub async fn get_player_games_for_user(
        db: &SqlitePool,
        user: &User,
        limit: i64,
    ) -> Result<Vec<PlayerGame>, sqlx::Error> {
        sqlx::query_as(
            "select players.game_id, players.player, players.dead, players.victory_click, players.top_score, players.score, games.start_time, games.end_time, games.rows, games.cols, games.num_mines, games.max_players from players left join games on players.game_id = games.game_id where players.user = ? order by games.start_time desc limit ?",
        )
        .bind(user.id)
        .bind(limit)
        .fetch_all(db)
        .await
    }

    pub async fn add_player(
        db: &SqlitePool,
        game_id: &str,
        user: &Option<User>,
        nickname: &Option<String>,
        player: u8,
    ) -> Result<(), sqlx::Error> {
        let id = user.as_ref().map(|u| u.id);
        sqlx::query(
            r#"
            insert into players (game_id, user, nickname, player)
            values (?, ?, ?, ?)
            "#,
        )
        .bind(game_id)
        .bind(id)
        .bind(nickname)
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
            sqlx::query("update players set dead = ?, score = ?, victory_click = ?, top_score = ? where game_id = ? and player = ?")
                .bind(p.dead)
                .bind(p.score as i64)
                .bind(p.victory_click)
                .bind(p.top_score)
                .bind(game_id)
                .bind(p.player_id as u8)
                .execute(&mut *transaction)
                .await?;
        }
        transaction.commit().await?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct GameLog {
    pub game_id: String,
    #[sqlx(json)]
    pub log: Vec<(Play, PlayOutcome)>,
}

impl GameLog {
    pub async fn get_log(db: &SqlitePool, game_id: &str) -> Result<Option<GameLog>, sqlx::Error> {
        sqlx::query_as("select * from game_log where game_id = ?")
            .bind(game_id)
            .fetch_optional(db)
            .await
    }

    pub async fn save_log(
        db: &SqlitePool,
        game_id: &str,
        log: Vec<(Play, PlayOutcome)>,
    ) -> Result<GameLog, sqlx::Error> {
        sqlx::query_as(
            r#"
            insert into game_log (game_id, log)
            values (?, ?)
            returning *
            "#,
        )
        .bind(game_id)
        .bind(Json(log))
        .fetch_one(db)
        .await
    }
}
