use anyhow::Result;
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use wasm_bindgen::prelude::*;

use minesweeper_lib::{
    board::{Board, BoardPoint, CompactBoard},
    cell::{HiddenCell, PlayerCell},
    client::MinesweeperClient,
    game::{
        Action as PlayAction, CompletedMinesweeper, Minesweeper, MinesweeperBuilder,
        MinesweeperOpts, Play,
    },
};

use game_ui::GameInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

#[derive(Clone)]
pub struct FrontendGame {
    pub err_signal: WriteSignal<Option<String>>,
    pub completed: ReadSignal<bool>,
    pub victory: ReadSignal<bool>,
    pub dead: ReadSignal<bool>,
    pub flag_count: ReadSignal<usize>,
    pub sync_time: ReadSignal<Option<usize>>,
    pub cells: Arc<Vec<Vec<ReadSignal<PlayerCell>>>>,
    pub start_time: ReadSignal<Option<DateTime<Utc>>>,
    cell_signals: Arc<Vec<Vec<WriteSignal<PlayerCell>>>>,
    set_completed: WriteSignal<bool>,
    set_victory: WriteSignal<bool>,
    set_dead: WriteSignal<bool>,
    set_flag_count: WriteSignal<usize>,
    set_sync_time: WriteSignal<Option<usize>>,
    set_start_time: WriteSignal<Option<DateTime<Utc>>>,
    game: Arc<RwLock<Minesweeper>>,
    game_client: Arc<RwLock<MinesweeperClient>>,
}

impl FrontendGame {
    pub fn new(game_info: &GameInfo, err_signal: WriteSignal<Option<String>>) -> Self {
        let (read_signals, write_signals) = signals_from_board(&game_info.board());
        let (completed, set_completed) = signal(game_info.is_completed);
        let (victory, set_victory) = signal(false);
        let (dead, set_dead) = signal(false);
        let (flag_count, set_flag_count) = signal(0);
        let (sync_time, set_sync_time) = signal::<Option<usize>>(None);
        let (start_time, set_start_time) = signal(game_info.start_time);
        let rows = game_info.rows;
        let cols = game_info.cols;
        let num_mines = game_info.num_mines;
        FrontendGame {
            cells: read_signals.into(),
            cell_signals: write_signals.into(),
            err_signal,
            completed,
            victory,
            dead,
            set_completed,
            set_victory,
            set_dead,
            flag_count,
            set_flag_count,
            sync_time,
            set_sync_time,
            start_time,
            set_start_time,
            game: Arc::new(RwLock::new(
                MinesweeperBuilder::new(MinesweeperOpts {
                    rows,
                    cols,
                    num_mines,
                })
                .expect("Should be able to create game")
                .with_log()
                .with_superclick()
                .init(),
            )),
            game_client: Arc::new(RwLock::new(MinesweeperClient::new(rows, cols))),
        }
    }

    pub fn try_reveal(&self, row: usize, col: usize) -> Result<()> {
        let point = BoardPoint { row, col };
        let play = Play {
            player: 0,
            action: PlayAction::Reveal,
            point,
        };
        self.try_play(play)
    }

    pub fn try_flag(&self, row: usize, col: usize) -> Result<()> {
        let point = BoardPoint { row, col };
        let play = Play {
            player: 0,
            action: PlayAction::Flag,
            point,
        };
        self.try_play(play)
    }

    pub fn try_reveal_adjacent(&self, row: usize, col: usize) -> Result<()> {
        let point = BoardPoint { row, col };
        let play = Play {
            player: 0,
            action: PlayAction::RevealAdjacent,
            point,
        };
        let res = self.try_play(play);
        match &res {
            Ok(_) => self.err_signal.set(None),
            Err(e) => self.err_signal.set(Some(format!("{e:?}"))),
        };
        res
    }

    fn try_play(&self, play: Play) -> Result<()> {
        let is_started = self.sync_time.get_untracked().is_some();
        let game_client: &mut MinesweeperClient = &mut (*self.game_client).write().unwrap();
        let game: &mut Minesweeper = &mut (*self.game).write().unwrap();
        let po = game.play(play)?;
        let plays = game_client.update(po);
        game_client.add_or_update_player(0, game.player_score(0).ok(), game.player_dead(0).ok());
        if !is_started {
            self.set_sync_time.set(Some(0));
            self.set_start_time.set(Some(Utc::now()));
        }
        plays.iter().for_each(|(point, cell)| {
            log::debug!("Play outcome: {point:?} {cell:?}");
            self.update_cell(*point, *cell);
        });
        let is_victory = game_client.victory;
        let is_dead = matches!(game.player_dead(0), Ok(true));

        if is_victory {
            self.set_victory.set(true);
            self.set_completed.set(true);
        } else if is_dead {
            self.set_dead.set(true);
            self.set_completed.set(true);
        }
        Ok(())
    }

    pub fn update_cell(&self, point: BoardPoint, cell: PlayerCell) {
        let curr_cell = self.cells[point.row][point.col].get_untracked();
        match (curr_cell, cell) {
            (PlayerCell::Hidden(HiddenCell::Flag), PlayerCell::Hidden(HiddenCell::Empty)) => {
                self.set_flag_count.update(|nm| *nm -= 1);
                log::debug!("Removed flag")
            }
            (PlayerCell::Hidden(HiddenCell::Flag), PlayerCell::Revealed(_)) => {
                self.set_flag_count.update(|nm| *nm -= 1);
                log::debug!("Removed flag")
            }
            (PlayerCell::Hidden(HiddenCell::Empty), PlayerCell::Hidden(HiddenCell::Flag)) => {
                self.set_flag_count.update(|nm| *nm += 1);
                log::debug!("Added flag")
            }
            (PlayerCell::Hidden(HiddenCell::Empty), PlayerCell::Revealed(rc))
                if rc.contents.is_mine() =>
            {
                self.set_flag_count.update(|nm| *nm += 1);
                log::debug!("Mine revealed")
            }
            _ => {}
        }
        self.cell_signals[point.row][point.col].set(cell);
    }

    pub fn extract_completed_game(&self) -> Option<CompletedMinesweeper> {
        // Check if game is over before attempting to consume it
        log::debug!("Checking if game is over before extracting completed game");
        {
            let game = self.game.read().ok()?;
            if !game.is_over() {
                log::debug!("Game is not over, cannot extract completed game");
                return None;
            }
        }

        log::debug!("Game is over, extracting completed game");
        // Take ownership of the game by replacing it with a dummy game
        // This is a bit of a hack, but necessary since complete() requires ownership
        let dummy_game = MinesweeperBuilder::new(MinesweeperOpts {
            rows: 1,
            cols: 2,
            num_mines: 1,
        })
        .ok()?
        .with_log()
        .init();

        log::debug!("Replacing game with dummy game to extract completed game");

        let mut game_lock = self.game.write().ok()?;
        log::debug!("Acquired write lock on game");
        let owned_game = std::mem::replace(&mut *game_lock, dummy_game);
        drop(game_lock);

        Some(owned_game.complete())
    }

    pub async fn save_game_with_completed(
        game_info: &GameInfo,
        is_completed: bool,
        victory: bool,
        start_time: Option<DateTime<Utc>>,
        completed_game: Option<Arc<CompletedMinesweeper>>,
    ) -> Result<(), String> {
        if !is_completed {
            return Err("Cannot save game that is not completed".to_string());
        }

        let game_id = format!("game_{}", chrono::Utc::now().timestamp());
        let rows = game_info.rows as u32;
        let cols = game_info.cols as u32;
        let num_mines = game_info.num_mines as u32;

        // Serialize the final board and game log if we have the completed game
        let (final_board, game_log) = if let Some(completed) = completed_game {
            let board = completed.viewer_board_final();
            let compact_board = CompactBoard::from_board(&board);
            let serialized_board = serde_json::to_string(&compact_board)
                .map_err(|e| format!("Failed to serialize board: {e}"))?;

            let log = completed.get_log();
            let compressed_log = log.map(|log| minesweeper_lib::game::compress_game_log(&log));

            (Some(serialized_board), compressed_log)
        } else {
            (None, None)
        };

        let saved_game = SavedGame {
            game_id,
            rows,
            cols,
            num_mines,
            is_completed,
            victory,
            start_time: start_time.map(|t| t.to_rfc3339()),
            end_time: Some(chrono::Utc::now().to_rfc3339()),
            final_board,
            game_log,
        };

        // Wrap the saved_game in an object with "game" key as expected by the Tauri command
        let args = js_sys::Object::new();
        js_sys::Reflect::set(
            &args,
            &JsValue::from_str("game"),
            &serde_wasm_bindgen::to_value(&saved_game)
                .map_err(|e| format!("Failed to serialize game: {e}"))?,
        )
        .map_err(|_| "Failed to create args object")?;

        let result = invoke("save_game", args.into()).await;

        // The result from Tauri is already unwrapped, so we just need to check if it's null (success) or a string (error)
        if result.is_null() || result.is_undefined() {
            Ok(())
        } else if let Ok(error_msg) = serde_wasm_bindgen::from_value::<String>(result) {
            Err(error_msg)
        } else {
            // If we can't parse as string, assume success
            Ok(())
        }
    }

    pub async fn get_saved_games() -> Result<Vec<SavedGame>, String> {
        let result = invoke("get_saved_games", JsValue::null()).await;

        // Log the raw result for debugging
        log::info!("Raw result from get_saved_games: {result:?}");

        match serde_wasm_bindgen::from_value::<Vec<SavedGame>>(result) {
            Ok(games) => {
                log::info!("Successfully deserialized {} games", games.len());
                Ok(games)
            }
            Err(e) => {
                log::error!("Failed to deserialize games: {e}");
                Err(format!("Failed to deserialize games: {e}"))
            }
        }
    }

    pub fn reconstruct_completed_game(
        saved_game: &SavedGame,
    ) -> Result<Option<CompletedMinesweeper>, String> {
        // Check if we have both the final board and game log
        let final_board_json = saved_game
            .final_board
            .as_ref()
            .ok_or("No final board data")?;
        let game_log_compressed = saved_game.game_log.as_ref().ok_or("No game log data")?;

        // Deserialize the final board
        let compact_board: CompactBoard = serde_json::from_str(final_board_json)
            .map_err(|e| format!("Failed to deserialize final board: {e}"))?;
        let final_board = compact_board.to_board();

        // Decompress the game log
        let game_log = minesweeper_lib::game::decompress_game_log(game_log_compressed);

        // Create a mock player for single-player games
        let player = minesweeper_lib::client::ClientPlayer {
            player_id: 0,
            username: String::new(),
            dead: !saved_game.victory,
            victory_click: saved_game.victory,
            top_score: false,
            score: 0,
        };

        // Reconstruct the CompletedMinesweeper
        let completed_game = CompletedMinesweeper::from_log(final_board, game_log, vec![player]);

        Ok(Some(completed_game))
    }

    pub async fn get_aggregate_stats() -> Result<AggregateStats, String> {
        let result = invoke("get_aggregate_stats", JsValue::null()).await;

        match serde_wasm_bindgen::from_value::<AggregateStats>(result) {
            Ok(stats) => {
                log::info!("Successfully fetched aggregate stats");
                Ok(stats)
            }
            Err(e) => {
                log::error!("Failed to fetch aggregate stats: {e}");
                Err(format!("Failed to fetch aggregate stats: {e}"))
            }
        }
    }

    pub async fn get_timeline_stats() -> Result<TimelineStats, String> {
        let result = invoke("get_timeline_stats", JsValue::null()).await;

        match serde_wasm_bindgen::from_value::<TimelineStats>(result) {
            Ok(stats) => {
                log::info!("Successfully fetched timeline stats");
                Ok(stats)
            }
            Err(e) => {
                log::error!("Failed to fetch timeline stats: {e}");
                Err(format!("Failed to fetch timeline stats: {e}"))
            }
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn signals_from_board(
    board: &Board<PlayerCell>,
) -> (
    Vec<Vec<ReadSignal<PlayerCell>>>,
    Vec<Vec<WriteSignal<PlayerCell>>>,
) {
    let mut read_signals = Vec::with_capacity(board.size());
    let mut write_signals = Vec::with_capacity(board.size());
    board.rows_iter().for_each(|cells| {
        let mut read_row = Vec::new();
        let mut write_row = Vec::new();
        cells.iter().for_each(|cell| {
            let (rs, ws) = signal(*cell);
            read_row.push(rs);
            write_row.push(ws);
        });
        read_signals.push(read_row);
        write_signals.push(write_row);
    });
    (read_signals, write_signals)
}
