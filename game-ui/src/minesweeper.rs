use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

use minesweeper_lib::{
    board::Board,
    cell::PlayerCell,
    client::ClientPlayer,
    game::{Play, PlayOutcome},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    pub game_id: String,
    pub has_owner: bool,
    pub is_owner: bool,
    pub rows: usize,
    pub cols: usize,
    pub num_mines: usize,
    pub max_players: u8,
    pub is_started: bool,
    pub is_completed: bool,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub final_board: Board<PlayerCell>,
    pub players: Vec<Option<ClientPlayer>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfoWithLog {
    pub game_info: GameInfo,
    pub player_num: Option<u8>,
    pub log: Vec<(Play, PlayOutcome)>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameSettings {
    pub rows: i64,
    pub cols: i64,
    pub num_mines: i64,
    pub max_players: i64,
}

impl GameSettings {
    pub fn new(rows: i64, cols: i64, num_mines: i64, max_players: i64) -> Self {
        Self {
            rows,
            cols,
            num_mines,
            max_players,
        }
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            rows: 50,
            cols: 50,
            num_mines: 500,
            max_players: 8,
        }
    }
}

impl From<&GameInfo> for GameSettings {
    fn from(value: &GameInfo) -> Self {
        GameSettings {
            rows: value.rows as i64,
            cols: value.cols as i64,
            num_mines: value.num_mines as i64,
            max_players: value.max_players as i64,
        }
    }
}

impl From<GameInfo> for GameSettings {
    fn from(value: GameInfo) -> Self {
        GameSettings::from(&value)
    }
}
