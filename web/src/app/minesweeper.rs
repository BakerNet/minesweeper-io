mod cell;
mod client;
mod entry;
mod game;
mod players;
mod widgets;

pub use entry::JoinOrCreateGame;
pub use game::Game;

use serde::{Deserialize, Serialize};

use minesweeper_lib::{cell::PlayerCell, client::ClientPlayer};

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
    final_board: Option<Vec<Vec<PlayerCell>>>,
    players: Vec<Option<ClientPlayer>>,
}

#[cfg(feature = "ssr")]
impl From<&PlayerUser> for ClientPlayer {
    fn from(value: &PlayerUser) -> Self {
        ClientPlayer {
            player_id: value.player as usize,
            username: FrontendUser::display_name_or_anon(&value.display_name, value.user.is_some()),
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
