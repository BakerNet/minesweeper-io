use chrono::{DateTime, TimeDelta, Utc};
use minesweeper_lib::{
    cell::PlayerCell,
    client::ClientPlayer,
    game::{Play, PlayOutcome},
};
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow, SqlitePool};

use web_auth::{models::User, FrontendUser};

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
    pub timed_out: Option<bool>,
    pub seconds: Option<i64>,
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
    pub timed_out: Option<bool>,
    pub seconds: Option<i64>,
    pub num_players: u8,
    pub top_score: Option<i64>,
}

pub struct GameParameters {
    pub rows: i64,
    pub cols: i64,
    pub num_mines: i64,
    pub max_players: u8,
}

impl Game {
    pub async fn get_game(db: &SqlitePool, game_id: &str) -> Result<Option<Game>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM games WHERE game_id = ?")
            .bind(game_id)
            .fetch_optional(db)
            .await
    }

    pub async fn get_games_with_players_by_ids<T>(
        db: &SqlitePool,
        game_ids: &[T],
    ) -> Result<Vec<SimpleGameWithPlayers>, sqlx::Error>
    where
        T: AsRef<str> + Send + Sync,
    {
        let params = format!("?{}", ", ?".repeat(game_ids.len() - 1));
        let query_str = format!(
            r#"
            SELECT
              game_id, owner, rows, cols, num_mines, max_players, is_completed, is_started, start_time, end_time, timed_out, seconds,
              ( SELECT count(*) FROM players WHERE players.game_id = games.game_id ) as num_players,
              ( SELECT max(score) FROM players WHERE players.game_id = games.game_id ) as top_score
            FROM games
            WHERE game_id IN ( {params} )
            ORDER BY start_time DESC
            LIMIT 100
            "#
        );

        let mut query = sqlx::query_as(&query_str);
        for i in game_ids {
            query = query.bind(i.as_ref());
        }
        query.fetch_all(db).await
    }

    pub async fn get_recent_games_with_players(
        db: &SqlitePool,
        duration: TimeDelta,
    ) -> Result<Vec<SimpleGameWithPlayers>, sqlx::Error> {
        let params = if duration.num_minutes().abs() > 0 {
            format!("{} minutes", duration.num_minutes())
        } else {
            format!("{} seconds", duration.num_seconds())
        };
        let query_str = format!(
            r#"
            SELECT 
              game_id, owner, rows, cols, num_mines, max_players, is_completed, is_started, start_time, end_time, timed_out, seconds,
              ( SELECT count(*) FROM players WHERE players.game_id = games.game_id ) as num_players,
              ( SELECT max(score) FROM players WHERE players.game_id = games.game_id ) as top_score
            FROM games
            WHERE is_completed = 1 AND start_time >= Datetime('now', '{params}')
            ORDER BY start_time DESC
            LIMIT 20
            "#
        );

        let query = sqlx::query_as(&query_str);
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
            INSERT INTO games (game_id, owner, rows, cols, num_mines, max_players, final_board)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            RETURNING *
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
        sqlx::query("UPDATE games SET is_started = 1 WHERE game_id = ?")
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
        sqlx::query("UPDATE games SET start_time = ? WHERE game_id = ?")
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
        sqlx::query("UPDATE games SET final_board = ? WHERE game_id = ?")
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
        end_time: Option<DateTime<Utc>>,
        seconds: Option<i64>,
        timed_out: bool,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE games
            SET
              is_completed = 1,
              final_board = ?,
              end_time = ?,
              timed_out = ?,
              seconds = ?
            WHERE game_id = ?
            "#,
        )
        .bind(Json(final_board))
        .bind(end_time)
        .bind(timed_out)
        .bind(seconds)
        .bind(game_id)
        .execute(db)
        .await
        .map(|_| ())
    }

    pub async fn set_all_games_completed(db: &SqlitePool) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE games SET is_completed = 1")
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

impl From<&PlayerUser> for ClientPlayer {
    fn from(value: &PlayerUser) -> Self {
        ClientPlayer {
            player_id: value.player as usize,
            username: FrontendUser::display_name_or_anon(
                value.display_name.as_ref(),
                value.user.is_some(),
            ),
            dead: value.dead,
            victory_click: value.victory_click,
            top_score: value.top_score,
            score: value.score as usize,
        }
    }
}

impl From<PlayerUser> for ClientPlayer {
    fn from(value: PlayerUser) -> Self {
        ClientPlayer::from(&value)
    }
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

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct GameStats {
    pub played: i64,
    pub best_time: i64,
    pub average_time: f64,
    pub victories: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AggregateStats {
    pub beginner: GameStats,
    pub intermediate: GameStats,
    pub expert: GameStats,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimelineStats {
    pub beginner: Vec<(bool, i64)>,
    pub intermediate: Vec<(bool, i64)>,
    pub expert: Vec<(bool, i64)>,
}

pub enum GameStatusFilter {
    All,
    Won,
    Lost,
    InProgress,
}

pub enum GameModeFilter {
    All,
    Beginner,
    Intermediate,
    Expert,
    Custom,
}

pub enum SortBy {
    Date,
    Duration,
}

pub enum SortOrder {
    Asc,
    Desc,
}

pub struct GameQueryParams {
    pub page: i64,
    pub limit: i64,
    pub mode_filter: GameModeFilter,
    pub status_filter: GameStatusFilter,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
}

impl Player {
    pub async fn get_players(
        db: &SqlitePool,
        game_id: &str,
    ) -> Result<Vec<PlayerUser>, sqlx::Error> {
        sqlx::query_as(
            r#"
            SELECT
               players.*,
               users.display_name
            FROM players
            LEFT JOIN users ON players.user = users.id
            WHERE players.game_id = ?
            "#,
        )
        .bind(game_id)
        .fetch_all(db)
        .await
    }

    pub async fn get_player_games_for_user(
        db: &SqlitePool,
        user: &User,
        params: &GameQueryParams,
    ) -> Result<Vec<PlayerGame>, sqlx::Error> {
        let mut where_clauses = vec!["players.user = ?".to_string()];

        // Add mode filter
        match params.mode_filter {
            GameModeFilter::Beginner => {
                where_clauses
                    .push("games.rows = 9 AND games.cols = 9 AND games.num_mines = 10".to_string());
            }
            GameModeFilter::Intermediate => {
                where_clauses.push(
                    "games.rows = 16 AND games.cols = 16 AND games.num_mines = 40".to_string(),
                );
            }
            GameModeFilter::Expert => {
                where_clauses.push(
                    "games.rows = 16 AND games.cols = 30 AND games.num_mines = 99".to_string(),
                );
            }
            GameModeFilter::Custom => {
                where_clauses.push("NOT (games.rows = 9 AND games.cols = 9 AND games.num_mines = 10) AND NOT (games.rows = 16 AND games.cols = 16 AND games.num_mines = 40) AND NOT (games.rows = 16 AND games.cols = 30 AND games.num_mines = 99)".to_string());
            }
            GameModeFilter::All => {}
        }

        // Add status filter
        match params.status_filter {
            GameStatusFilter::Won => {
                where_clauses.push("players.victory_click = 1".to_string());
            }
            GameStatusFilter::Lost => {
                where_clauses.push("players.dead = 1".to_string());
            }
            GameStatusFilter::InProgress => {
                where_clauses.push("games.is_completed = 0".to_string());
            }
            GameStatusFilter::All => {}
        }

        // Build ORDER BY clause
        let order_by = match (&params.sort_by, &params.sort_order) {
            (SortBy::Date, SortOrder::Desc) => "ORDER BY games.start_time DESC",
            (SortBy::Date, SortOrder::Asc) => "ORDER BY games.start_time ASC",
            (SortBy::Duration, SortOrder::Desc) => "ORDER BY games.seconds DESC",
            (SortBy::Duration, SortOrder::Asc) => "ORDER BY games.seconds ASC",
        };

        let offset = (params.page - 1) * params.limit;
        let where_clause = format!("WHERE {}", where_clauses.join(" AND "));

        let query_str = format!(
            r#"
            SELECT
              players.game_id, players.player, players.dead, players.victory_click, players.top_score, players.score,
              games.start_time, games.end_time, games.rows, games.cols, games.num_mines, games.max_players
            FROM players
            LEFT JOIN games ON players.game_id = games.game_id
            {}
            {}
            LIMIT {} OFFSET {}
            "#,
            where_clause,
            order_by,
            params.limit + 1,
            offset
        );

        sqlx::query_as(&query_str).bind(user.id).fetch_all(db).await
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
            INSERT INTO players (game_id, user, nickname, player)
            VALUES (?, ?, ?, ?)
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
            sqlx::query("UPDATE players SET dead = ?, score = ?, victory_click = ?, top_score = ? WHERE game_id = ? AND player = ?")
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

    pub async fn get_aggregate_stats_for_user(
        db: &SqlitePool,
        user: &User,
    ) -> Result<AggregateStats, sqlx::Error> {
        let modes = [(9, 9, 10), (16, 16, 40), (16, 30, 99)];
        let mut queries = [String::new(), String::new(), String::new()];
        modes.into_iter().enumerate().for_each(|(i, mode)| {
            queries[i] = format!(
                r#"
                SELECT
                count(*) as played,
                sum(players.victory_click) as victories,
                min(games.seconds) FILTER (WHERE players.victory_click = 1) as best_time,
                avg(games.seconds) FILTER (WHERE players.victory_click = 1) as average_time
                FROM players
                LEFT JOIN games ON players.game_id = games.game_id
                WHERE 
                  players.user = ?
                  AND games.rows = {} AND games.cols = {} AND games.num_mines = {} AND games.max_players = 1 
                  AND games.seconds IS NOT NULL
                "#,
                mode.0,
                mode.1,
                mode.2
            );
        });

        Ok(AggregateStats {
            beginner: sqlx::query_as(&queries[0])
                .bind(user.id)
                .fetch_one(db)
                .await?,
            intermediate: sqlx::query_as(&queries[1])
                .bind(user.id)
                .fetch_one(db)
                .await?,
            expert: sqlx::query_as(&queries[2])
                .bind(user.id)
                .fetch_one(db)
                .await?,
        })
    }

    pub async fn get_timeline_stats_for_user(
        db: &SqlitePool,
        user: &User,
    ) -> Result<TimelineStats, sqlx::Error> {
        let modes = [(9, 9, 10), (16, 16, 40), (16, 30, 99)];
        let mut queries = [String::new(), String::new(), String::new()];
        modes.into_iter().enumerate().for_each(|(i, mode)| {
            queries[i] = format!(
                r#"
                SELECT
                players.victory_click,
                games.seconds
                FROM players
                LEFT JOIN games ON players.game_id = games.game_id
                WHERE 
                  players.user = ?
                  AND games.rows = {} AND games.cols = {} AND games.num_mines = {} AND games.max_players = 1 
                  AND games.seconds IS NOT NULL
                ORDER BY games.start_time desc
                LIMIT 1000
                "#,
                mode.0,
                mode.1,
                mode.2
            );
        });

        Ok(TimelineStats {
            beginner: sqlx::query_as(&queries[0])
                .bind(user.id)
                .fetch_all(db)
                .await?,
            intermediate: sqlx::query_as(&queries[1])
                .bind(user.id)
                .fetch_all(db)
                .await?,
            expert: sqlx::query_as(&queries[2])
                .bind(user.id)
                .fetch_all(db)
                .await?,
        })
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
        sqlx::query_as("SELECT * FROM game_log WHERE game_id = ?")
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
            INSERT INTO game_log (game_id, log)
            VALUES (?, ?)
            RETURNING *
            "#,
        )
        .bind(game_id)
        .bind(Json(log))
        .fetch_one(db)
        .await
    }
}
