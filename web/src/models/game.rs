use cfg_if::cfg_if;

use minesweeper::cell::PlayerCell;
use serde::{Deserialize, Serialize};

cfg_if! { if #[cfg(feature="ssr")] {
    use sqlx::FromRow;
}}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Game {
    pub game_id: String,
    pub owner: i64, // User.id
    pub rows: usize,
    pub cols: usize,
    pub num_mines: usize,
    pub max_players: usize,
    pub is_completed: bool,
    pub final_board: Vec<Vec<PlayerCell>>,
}

// Here we've implemented `Debug` manually to avoid accidentally logging the
// access token.
impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Game")
            .field("id", &self.game_id)
            .field("owner", &self.owner)
            .field("is_completed", &self.is_completed)
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(FromRow))]
pub struct Player {
    game_id: String,
    pub user: i64, // User.id
    pub player: u8,
    pub dead: bool,
    pub score: u64,
}
