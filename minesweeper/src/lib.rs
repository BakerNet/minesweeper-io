use cell::PlayerCell;
use game::PlayOutcome;
use serde::{Deserialize, Serialize};

pub mod board;
pub mod cell;
pub mod client;
pub mod game;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMessage {
    PlayOutcome(PlayOutcome),
    GameState(Vec<Vec<PlayerCell>>),
    Error(String),
}
