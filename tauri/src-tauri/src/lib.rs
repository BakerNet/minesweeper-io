use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteConnectOptions, Row, SqlitePool};
use std::fs;
use std::str::FromStr;
use tauri::{command, AppHandle, Manager, State};
use tokio::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
pub struct SavedGame {
    pub game_id: String,
    pub rows: u32,
    pub cols: u32,
    pub num_mines: u32,
    pub is_completed: bool,
    pub victory: bool,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub final_board: Option<String>,
    pub game_log: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameStats {
    pub total_games: u32,
    pub wins: u32,
    pub losses: u32,
    pub win_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameModeStats {
    pub played: u32,
    pub victories: u32,
    pub best_time: Option<u32>,
    pub average_time: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregateStats {
    pub beginner: GameModeStats,
    pub intermediate: GameModeStats,
    pub expert: GameModeStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimelineGameData {
    pub victory: bool,
    pub seconds: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimelineStats {
    pub beginner: Vec<TimelineGameData>,
    pub intermediate: Vec<TimelineGameData>,
    pub expert: Vec<TimelineGameData>,
}

struct DatabaseState {
    pool: Mutex<SqlitePool>,
}

#[command]
async fn save_game(state: State<'_, DatabaseState>, game: SavedGame) -> Result<(), String> {
    let pool = state.pool.lock().await;

    sqlx::query(
        "INSERT OR REPLACE INTO games (game_id, rows, cols, num_mines, is_completed, victory, start_time, end_time, final_board, game_log) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&game.game_id)
    .bind(game.rows as i64)
    .bind(game.cols as i64)
    .bind(game.num_mines as i64)
    .bind(game.is_completed as i32)
    .bind(game.victory as i32)
    .bind(&game.start_time.unwrap_or_default())
    .bind(&game.end_time.unwrap_or_default())
    .bind(&game.final_board.unwrap_or_default())
    .bind(&game.game_log)
    .execute(&*pool)
    .await
    .map_err(|e| format!("Failed to save game: {}", e))?;

    Ok(())
}

async fn get_game_mode_stats(
    pool: &SqlitePool,
    rows: u32,
    cols: u32,
    num_mines: u32,
) -> Result<GameModeStats, String> {
    let row = sqlx::query(
        "SELECT 
            COUNT(*) as played,
            SUM(CASE WHEN victory = 1 THEN 1 ELSE 0 END) as victories,
            MIN(CASE WHEN victory = 1 AND start_time IS NOT NULL AND end_time IS NOT NULL 
                THEN (julianday(end_time) - julianday(start_time)) * 86400 
                ELSE NULL END) as best_time,
            AVG(CASE WHEN victory = 1 AND start_time IS NOT NULL AND end_time IS NOT NULL 
                THEN (julianday(end_time) - julianday(start_time)) * 86400 
                ELSE NULL END) as average_time
         FROM games 
         WHERE is_completed = 1 
           AND rows = ? AND cols = ? AND num_mines = ?
           AND start_time IS NOT NULL AND end_time IS NOT NULL",
    )
    .bind(rows as i64)
    .bind(cols as i64)
    .bind(num_mines as i64)
    .fetch_one(pool)
    .await
    .map_err(|e| format!("Failed to get game mode stats: {}", e))?;

    let played: i64 = row.try_get("played").unwrap_or(0);
    let victories: i64 = row.try_get("victories").unwrap_or(0);
    let best_time: Option<f64> = row.try_get("best_time").ok();
    let average_time: Option<f64> = row.try_get("average_time").ok();

    Ok(GameModeStats {
        played: played as u32,
        victories: victories as u32,
        best_time: best_time.map(|t| t.floor() as u32),
        average_time,
    })
}

#[command]
async fn get_aggregate_stats(state: State<'_, DatabaseState>) -> Result<AggregateStats, String> {
    let pool = state.pool.lock().await;

    let beginner = get_game_mode_stats(&pool, 9, 9, 10).await?;
    let intermediate = get_game_mode_stats(&pool, 16, 16, 40).await?;
    let expert = get_game_mode_stats(&pool, 16, 30, 99).await?;

    Ok(AggregateStats {
        beginner,
        intermediate,
        expert,
    })
}

async fn get_game_mode_timeline(
    pool: &SqlitePool,
    rows: u32,
    cols: u32,
    num_mines: u32,
) -> Result<Vec<TimelineGameData>, String> {
    let timeline_rows = sqlx::query(
        "SELECT 
            victory,
            (julianday(end_time) - julianday(start_time)) * 86400 as seconds
         FROM games 
         WHERE is_completed = 1 
           AND rows = ? AND cols = ? AND num_mines = ?
           AND start_time IS NOT NULL AND end_time IS NOT NULL
         ORDER BY start_time ASC
         LIMIT 1000",
    )
    .bind(rows as i64)
    .bind(cols as i64)
    .bind(num_mines as i64)
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Failed to get timeline stats: {}", e))?;

    let mut timeline_data = Vec::new();
    for row in timeline_rows {
        let victory: i32 = row.try_get("victory").unwrap_or(0);
        let seconds: f64 = row.try_get("seconds").unwrap_or(0.0);

        timeline_data.push(TimelineGameData {
            victory: victory != 0,
            seconds: seconds.floor() as u32,
        });
    }

    Ok(timeline_data)
}

#[command]
async fn get_timeline_stats(state: State<'_, DatabaseState>) -> Result<TimelineStats, String> {
    let pool = state.pool.lock().await;

    let beginner = get_game_mode_timeline(&pool, 9, 9, 10).await?;
    let intermediate = get_game_mode_timeline(&pool, 16, 16, 40).await?;
    let expert = get_game_mode_timeline(&pool, 16, 30, 99).await?;

    Ok(TimelineStats {
        beginner,
        intermediate,
        expert,
    })
}

#[command]
async fn get_game_stats(state: State<'_, DatabaseState>) -> Result<GameStats, String> {
    let pool = state.pool.lock().await;

    let row = sqlx::query(
        "SELECT 
            COUNT(*) as total_games,
            SUM(CASE WHEN victory = 1 THEN 1 ELSE 0 END) as wins,
            SUM(CASE WHEN victory = 0 AND is_completed = 1 THEN 1 ELSE 0 END) as losses
         FROM games WHERE is_completed = 1",
    )
    .fetch_one(&*pool)
    .await
    .map_err(|e| format!("Failed to get stats: {}", e))?;

    let total_games: i64 = row.try_get("total_games").unwrap_or(0);
    let wins: i64 = row.try_get("wins").unwrap_or(0);
    let losses: i64 = row.try_get("losses").unwrap_or(0);
    let win_rate = if total_games > 0 {
        wins as f64 / total_games as f64
    } else {
        0.0
    };

    Ok(GameStats {
        total_games: total_games as u32,
        wins: wins as u32,
        losses: losses as u32,
        win_rate,
    })
}

#[command]
async fn get_saved_games(state: State<'_, DatabaseState>) -> Result<Vec<SavedGame>, String> {
    let pool = state.pool.lock().await;

    let rows = sqlx::query(
        "SELECT game_id, rows, cols, num_mines, is_completed, victory, start_time, end_time, final_board, game_log
         FROM games ORDER BY start_time DESC LIMIT 100"
    )
    .fetch_all(&*pool)
    .await
    .map_err(|e| format!("Failed to get saved games: {}", e))?;

    let mut games = Vec::new();
    for row in rows {
        let game_log: Option<Vec<u8>> = match row.try_get("game_log") {
            Ok(blob) => blob,
            Err(_) => None,
        };

        games.push(SavedGame {
            game_id: row.try_get("game_id").unwrap_or_default(),
            rows: row.try_get::<i64, _>("rows").unwrap_or(0) as u32,
            cols: row.try_get::<i64, _>("cols").unwrap_or(0) as u32,
            num_mines: row.try_get::<i64, _>("num_mines").unwrap_or(0) as u32,
            is_completed: row.try_get::<i32, _>("is_completed").unwrap_or(0) != 0,
            victory: row.try_get::<i32, _>("victory").unwrap_or(0) != 0,
            start_time: row.try_get("start_time").ok(),
            end_time: row.try_get("end_time").ok(),
            final_board: row.try_get("final_board").ok(),
            game_log,
        });
    }

    println!("Returning {} games", games.len());
    for (i, game) in games.iter().enumerate() {
        println!(
            "Game {}: {} ({}x{}, {} mines)",
            i, game.game_id, game.rows, game.cols, game.num_mines
        );
    }

    Ok(games)
}

#[command]
async fn load_game_replay(
    state: State<'_, DatabaseState>,
    game_id: String,
) -> Result<Option<Vec<u8>>, String> {
    let pool = state.pool.lock().await;

    let row = sqlx::query("SELECT game_log FROM games WHERE game_id = ? AND game_log IS NOT NULL")
        .bind(&game_id)
        .fetch_optional(&*pool)
        .await
        .map_err(|e| format!("Failed to get game replay: {}", e))?;

    if let Some(row) = row {
        let game_log: Option<Vec<u8>> = row.try_get("game_log").ok();
        Ok(game_log)
    } else {
        Ok(None)
    }
}

async fn establish_connection(app_handle: AppHandle) -> SqlitePool {
    // Get the app data directory
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data directory");

    // Create the directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&app_data_dir) {
        eprintln!("Failed to create app data directory: {}", e);
    }

    // Create the full path to the database file
    let db_file_path = app_data_dir.join("minesweeper.db");

    println!("Connecting to database at: {}", db_file_path.display());

    // Use SqliteConnectOptions to enable creating the database file if it doesn't exist
    let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_file_path.display()))
        .expect("Invalid database URL")
        .create_if_missing(true);

    let pool = SqlitePool::connect_with(options)
        .await
        .expect("Failed to connect to database");

    // Create the games table if it doesn't exist
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS games (
            game_id TEXT PRIMARY KEY,
            rows INTEGER NOT NULL,
            cols INTEGER NOT NULL,
            num_mines INTEGER NOT NULL,
            is_completed INTEGER NOT NULL DEFAULT 0,
            victory INTEGER NOT NULL DEFAULT 0,
            start_time TEXT,
            end_time TEXT,
            final_board TEXT
        )",
    )
    .execute(&pool)
    .await
    .expect("Failed to create games table");

    // Add game_log column if it doesn't exist (migration)
    sqlx::query("ALTER TABLE games ADD COLUMN game_log BLOB")
        .execute(&pool)
        .await
        .ok(); // Ignore error if column already exists

    println!("Database initialized successfully!");

    pool
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            // Block on database initialization to ensure it's ready before the app starts
            tauri::async_runtime::block_on(async move {
                let pool = establish_connection(app_handle.clone()).await;
                app_handle.manage(DatabaseState {
                    pool: Mutex::new(pool),
                });
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            save_game,
            get_game_stats,
            get_saved_games,
            load_game_replay,
            get_aggregate_stats,
            get_timeline_stats,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
