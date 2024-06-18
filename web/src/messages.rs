use serde::{Deserialize, Serialize};

use minesweeper_lib::{cell::PlayerCell, client::ClientPlayer, game::PlayOutcome};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "game_message", content = "data")]
pub enum GameMessage {
    PlayerId(usize),
    PlayOutcome(PlayOutcome),
    PlayerUpdate(ClientPlayer),
    GameState(Vec<Vec<PlayerCell>>),
    PlayersState(Vec<Option<ClientPlayer>>),
    GameStarted,
    Error(String),
}

#[cfg(feature = "ssr")]
impl GameMessage {
    pub fn into_json(self) -> String {
        serde_json::to_string::<GameMessage>(&self)
            .unwrap_or_else(|_| panic!("Should be able to serialize GameMessage {:?}", self))
    }
}
