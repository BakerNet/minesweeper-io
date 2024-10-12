mod cell;
mod client;
mod entry;
mod game;
mod games;
mod players;
mod replay;
mod widgets;

use chrono::{DateTime, Utc};
pub use entry::{GameMode, JoinOrCreateGame};
pub use game::{GameView, GameWrapper, ReplayView};
pub use games::{ActiveGames, RecentGames};

use serde::{Deserialize, Serialize};

use minesweeper_lib::{
    board::Board,
    cell::PlayerCell,
    client::ClientPlayer,
    game::{Play, PlayOutcome},
};

#[cfg(feature = "ssr")]
use super::auth::FrontendUser;
#[cfg(feature = "ssr")]
use crate::models::game::PlayerUser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    game_id: String,
    has_owner: bool,
    is_owner: bool,
    rows: usize,
    cols: usize,
    num_mines: usize,
    max_players: u8,
    is_started: bool,
    is_completed: bool,
    start_time: Option<DateTime<Utc>>,
    end_time: Option<DateTime<Utc>>,
    final_board: Board<PlayerCell>,
    players: Vec<Option<ClientPlayer>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfoWithLog {
    game_info: GameInfo,
    player_num: Option<u8>,
    log: Vec<(Play, PlayOutcome)>,
}

#[cfg(feature = "ssr")]
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

#[cfg(feature = "ssr")]
impl From<PlayerUser> for ClientPlayer {
    fn from(value: PlayerUser) -> Self {
        ClientPlayer::from(&value)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameSettings {
    rows: i64,
    cols: i64,
    num_mines: i64,
    max_players: i64,
}

#[cfg(feature = "ssr")]
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
