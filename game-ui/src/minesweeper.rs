use chrono::{DateTime, Utc};

use serde::{Deserialize, Serialize};

use minesweeper_lib::{
    board::{Board, CompactBoard},
    cell::{HiddenCell, PlayerCell},
    client::ClientPlayer,
    game::{CompactPlayOutcome, Play, PlayOutcome},
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
    pub final_board: CompactBoard,
    pub players: Vec<Option<ClientPlayer>>,
}

impl GameInfo {
    pub fn new_singleplayer(game_id: String, rows: usize, cols: usize, num_mines: usize) -> Self {
        Self {
            game_id,
            has_owner: false,
            is_owner: false,
            rows,
            cols,
            num_mines,
            max_players: 1,
            is_started: true,
            is_completed: false,
            start_time: Some(Utc::now()),
            end_time: None,
            final_board: CompactBoard::from_board(&Board::new(
                rows,
                cols,
                PlayerCell::Hidden(HiddenCell::Empty),
            )),
            players: vec![None],
        }
    }

    /// Convert the compact board to full Board<PlayerCell> for UI use
    pub fn board(&self) -> Board<PlayerCell> {
        self.final_board.to_board()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfoWithLog {
    pub game_info: GameInfo,
    pub player_num: Option<u8>,
    pub log: Vec<(Play, CompactPlayOutcome)>,
}

impl GameInfoWithLog {
    /// Convert the compact log to full PlayOutcome format for use with CompletedMinesweeper
    pub fn full_log(&self) -> Vec<(Play, PlayOutcome)> {
        self.log
            .iter()
            .map(|(play, compact_outcome)| (*play, compact_outcome.to_full()))
            .collect()
    }
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
