use leptos::*;
use minesweeper_lib::{cell::PlayerCell, client::ClientPlayer, replay::MinesweeperReplay};

#[component]
#[allow(unused_variables)]
pub fn ReplayControls(
    replay: MinesweeperReplay,
    cell_write_signals: Vec<Vec<WriteSignal<PlayerCell>>>,
    set_flag_count: WriteSignal<usize>,
    player_write_signals: Vec<WriteSignal<Option<ClientPlayer>>>,
) -> impl IntoView {
    log::debug!("replay log length: {}", replay.len());
}
