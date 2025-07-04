pub mod background;
pub mod components;
pub mod dark_mode;
pub mod icons;
pub mod info;

mod game;
mod minesweeper;
mod mode;
mod players;
mod replay;
mod stats;
mod widgets;

pub use background::{AnimatedBackground, BackgroundToggle, BackgroundVariant};
pub use dark_mode::DarkModeToggle;
pub use game::{ActiveGame, InactiveGame, ReplayGame};
pub use minesweeper::{GameInfo, GameInfoWithLog, GameSettings};
pub use mode::{GameMode, PresetButtons, SettingsInputs};
pub use players::{ActivePlayers, InactivePlayers};
pub use replay::ReplayControls;
pub use stats::{
    parse_timeline_stats, PlayerGameModeStats, PlayerStats, PlayerStatsRow, PlayerStatsTable,
    TimelineGameModeStats, TimelineStats, TimelineStatsGraphs,
};
pub use widgets::{
    game_time_from_start_end, ActiveMines, ActiveTimer, CopyGameLink, GameStateWidget, GameWidgets, InactiveMines,
    InactiveTimer,
};

use leptos::prelude::*;

pub fn logo() -> impl IntoView {
    let white_bg = "bg-white hover:bg-neutral-300";
    let cell_class_1 = cell_class!(number_class!(1), white_bg);
    let cell_class_2 = cell_class!(number_class!(2), white_bg);
    let cell_class_3 = cell_class!(number_class!(3), white_bg);
    let cell_class_4 = cell_class!(number_class!(4), white_bg);
    let cell_class_flag = cell_class!("", "bg-neutral-500 hover:bg-neutral-600/90");
    view! {
        <span class="whitespace-nowrap">
            <span class=cell_class_4.clone()>M</span>
            <span class=cell_class_2.clone()>i</span>
            <span class=cell_class_3.clone()>n</span>
            <span class=cell_class_3>e</span>
            <span class=cell_class_4>s</span>
            <span class=cell_class_2.clone()>w</span>
            <span class=cell_class_2>e</span>
            <span class=cell_class_1.clone()>e</span>
            <span class=cell_class_flag>
                <icons::Flag />
            </span>
            <span class=cell_class_1.clone()>e</span>
            <span class=cell_class_1>r</span>
        </span>
    }
}
