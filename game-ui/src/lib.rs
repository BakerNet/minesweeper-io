pub mod components;
pub mod icons;

mod game;
mod minesweeper;
mod mode;
mod players;
mod replay;
mod widgets;

pub use game::{ActiveGame, InactiveGame, ReplayGame};
pub use minesweeper::{GameInfo, GameInfoWithLog, GameSettings};
pub use mode::{GameMode, PresetButtons, SettingsInputs};
pub use players::{ActivePlayers, InactivePlayers};
pub use replay::ReplayControls;
pub use widgets::{
    game_time_from_start_end, ActiveMines, ActiveTimer, CopyGameLink, GameWidgets, InactiveMines,
    InactiveTimer,
};
