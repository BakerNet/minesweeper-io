use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Error as SerdeJsonError;

use minesweeper_lib::{
    board::Board,
    cell::PlayerCell,
    client::ClientPlayer,
    game::{Play, PlayOutcome},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "game_message", content = "data")]
pub enum GameMessage {
    PlayerId(usize),
    PlayOutcome(PlayOutcome),
    PlayerUpdate(ClientPlayer),
    GameState(Board<PlayerCell>),
    PlayersState(Vec<Option<ClientPlayer>>),
    GameStarted,
    TopScore(usize),
    SyncTimer(usize),
    Error(String),
}

#[cfg(feature = "ssr")]
impl GameMessage {
    pub fn into_json(self) -> String {
        serde_json::to_string::<GameMessage>(&self)
            .unwrap_or_else(|_| panic!("Should be able to serialize GameMessage {:?}", self))
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
