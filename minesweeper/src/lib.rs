use cell::PlayerCell;
use client::ClientPlayer;
use game::PlayOutcome;
use serde::{Deserialize, Serialize};

pub mod board;
pub mod cell;
pub mod client;
pub mod game;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameMessage {
    PlayOutcome(PlayOutcome),
    PlayerUpdate(ClientPlayer),
    GameState(Vec<Vec<PlayerCell>>),
    PlayersState(Vec<Option<ClientPlayer>>),
    Error(String),
}
