use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeJsonError;

use minesweeper_lib::{
    board::CompactBoard,
    client::ClientPlayer,
    game::{Play, CompactPlayOutcome},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "game_message", content = "data")]
pub enum GameMessage {
    PlayerId(usize),
    PlayOutcome(CompactPlayOutcome),
    PlayerUpdate(ClientPlayer),
    GameState(CompactBoard),
    PlayersState(Vec<Option<ClientPlayer>>),
    GameStarted,
    TopScore(usize),
    SyncTimer(usize),
    Error(String),
}

impl GameMessage {
    pub fn into_json(self) -> String {
        serde_json::to_string::<GameMessage>(&self)
            .unwrap_or_else(|_| panic!("Should be able to serialize GameMessage {self:?}"))
    }
}

impl FromStr for GameMessage {
    type Err = SerdeJsonError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str::<GameMessage>(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "client_message", content = "data")]
pub enum ClientMessage {
    Join,
    PlayGame,
    Play(Play),
}
