use anyhow::Result;
use leptos::{html::Input, prelude::*};
use std::sync::{Arc, RwLock};

use crate::button_class;
use minesweeper_lib::{
    board::Board,
    cell::{HiddenCell, PlayerCell},
    client::ClientPlayer,
    game::Play,
    replay::{
        MinesweeperReplayWithAnalysis, ReplayAnalysisCell, ReplayPosition, Replayable, SimplePlayer,
    },
};

#[derive(Clone)]
struct ReplayStore {
    replay: Arc<RwLock<MinesweeperReplayWithAnalysis>>,
    cell_read_signals: Arc<Vec<Vec<ReadSignal<ReplayAnalysisCell>>>>,
    cell_write_signals: Arc<Vec<Vec<WriteSignal<ReplayAnalysisCell>>>>,
    player_write_signals: Option<Arc<Vec<WriteSignal<Option<ClientPlayer>>>>>,
}

impl ReplayStore {
    fn with_current_board(&self, f: impl FnOnce(&Board<ReplayAnalysisCell>)) {
        let replay: &MinesweeperReplayWithAnalysis = &mut (*self.replay).read().unwrap();
        f(replay.current_board())
    }

    fn with_current_players(&self, f: impl FnOnce(&Vec<SimplePlayer>)) {
        let replay: &MinesweeperReplayWithAnalysis = &mut (*self.replay).read().unwrap();
        f(replay.current_players())
    }

    fn next(&self) -> Result<ReplayPosition> {
        let replay: &mut MinesweeperReplayWithAnalysis = &mut (*self.replay).write().unwrap();
        replay.advance()
    }

    fn prev(&self) -> Result<ReplayPosition> {
        let replay: &mut MinesweeperReplayWithAnalysis = &mut (*self.replay).write().unwrap();
        replay.rewind()
    }

    fn to_pos(&self, pos: usize) -> Result<ReplayPosition> {
        let replay: &mut MinesweeperReplayWithAnalysis = &mut (*self.replay).write().unwrap();
        let pos = ReplayPosition::from_pos(pos, replay.len());
        replay.to_pos(pos)
    }

    fn flags(&self) -> usize {
        let replay: &mut MinesweeperReplayWithAnalysis = &mut (*self.replay).write().unwrap();
        replay.current_flags_and_revealed_mines()
    }

    fn current_play(&self) -> Option<Play> {
        let replay: &mut MinesweeperReplayWithAnalysis = &mut (*self.replay).write().unwrap();
        replay.current_play()
    }
}

#[component]
pub fn ReplayControls(
    replay: MinesweeperReplayWithAnalysis,
    cell_read_signals: Arc<Vec<Vec<ReadSignal<ReplayAnalysisCell>>>>,
    cell_write_signals: Arc<Vec<Vec<WriteSignal<ReplayAnalysisCell>>>>,
    flag_count_setter: Option<WriteSignal<usize>>,
    player_write_signals: Option<Arc<Vec<WriteSignal<Option<ClientPlayer>>>>>,
) -> impl IntoView {
    log::debug!("replay log length: {}", replay.len());
    let min = 0;
    let max = replay.len() - 1;
    let slider_el = NodeRef::<Input>::new();

    let (show_mines, set_show_mines) = signal(true);
    let (show_analysis, set_show_analysis) = signal(true);
    let (is_beginning, set_beginning) = signal(true);
    let (is_end, set_end) = signal(false);
    let (current_play, set_current_play) = signal::<Option<Play>>(None);

    let replay = ReplayStore {
        replay: Arc::new(RwLock::new(replay)),
        cell_read_signals,
        cell_write_signals,
        player_write_signals,
    };
    let replay = StoredValue::new(replay);

    let render_cell = move |replay: &ReplayStore,
                            row: usize,
                            col: usize,
                            ReplayAnalysisCell(pc, ac): &ReplayAnalysisCell| {
        let pc = if !show_mines.get_untracked() {
            match pc {
                PlayerCell::Hidden(HiddenCell::Mine) => PlayerCell::Hidden(HiddenCell::Empty),
                PlayerCell::Hidden(HiddenCell::FlagMine) => PlayerCell::Hidden(HiddenCell::Flag),
                default => *default,
            }
        } else {
            *pc
        };
        let ac = if !show_analysis.get_untracked() {
            None
        } else {
            *ac
        };
        let cell = ReplayAnalysisCell(pc, ac);
        if replay.cell_read_signals[row][col].get_untracked() != cell {
            replay.cell_write_signals[row][col](cell);
        }
    };
    let render_current = move || {
        replay.with_value(|replay| {
            replay.with_current_board(|current_board| {
                current_board
                    .rows_iter()
                    .enumerate()
                    .for_each(|(row, vec)| {
                        vec.iter()
                            .enumerate()
                            .for_each(|(col, cell)| render_cell(replay, row, col, cell))
                    })
            });
            if let Some(player_write_signals) = replay.player_write_signals.as_ref() {
                replay.with_current_players(|current_players| {
                    current_players.iter().enumerate().for_each(|(i, p)| {
                        player_write_signals[i].update(|cp| {
                            if let Some(cp) = cp.as_mut() {
                                p.update_client_player(cp);
                            }
                        });
                    })
                });
            }
            if let Some(set_flag_count) = flag_count_setter {
                set_flag_count(replay.flags());
            }
            set_current_play(replay.current_play());
        })
    };

    Effect::watch(
        show_mines,
        move |show_mines, _, prev| {
            if prev != Some(*show_mines) {
                render_current();
            }
            *show_mines
        },
        false,
    );

    Effect::watch(
        show_analysis,
        move |show_analysis, _, prev| {
            if prev != Some(*show_analysis) {
                render_current();
            }
            *show_analysis
        },
        false,
    );

    let next = move || {
        replay.with_value(|replay| {
            let res = replay.next();
            let slider = slider_el
                .get_untracked()
                .expect("Slider reference should be set");
            if let Ok(res) = &res {
                render_current();
                let new_pos = res.to_num(max);
                slider.set_value(&format!("{new_pos}"));
                if matches!(res, ReplayPosition::End) {
                    set_end(true);
                }
            }
            set_beginning(false);
        })
    };

    let prev = move || {
        replay.with_value(|replay| {
            let res = replay.prev();
            let slider = slider_el
                .get_untracked()
                .expect("Slider reference should be set");
            if let Ok(res) = &res {
                render_current();
                let new_pos = res.to_num(max);
                slider.set_value(&format!("{new_pos}"));
                if matches!(res, ReplayPosition::Beginning) {
                    set_beginning(true);
                }
            }
            set_end(false);
        })
    };

    let to_pos = move || {
        replay.with_value(|replay| {
            let slider = slider_el
                .get_untracked()
                .expect("Slider reference should be set")
                .value();
            let pos = slider
                .parse::<usize>()
                .expect("Slider value should be number");
            let res = replay.to_pos(pos);
            if res.is_ok() {
                render_current();
            }
            log::debug!("Slider: {pos} / {max}");
            match res {
                Ok(ReplayPosition::Beginning) => {
                    set_beginning(true);
                    set_end(false);
                }
                Ok(ReplayPosition::End) => {
                    set_beginning(false);
                    set_end(true);
                }
                _ => {
                    set_beginning(false);
                    set_end(false);
                }
            }
        })
    };

    render_current();

    view! {
        <div class="flex flex-col items-center space-y-2 mb-8">
            <div class="table border-separate border-spacing-2">
                <label class="table-row cursor-pointer">
                    <input
                        type="checkbox"
                        value=""
                        class="table-cell sr-only peer"
                        checked
                        on:change=move |ev| {
                            set_show_mines(event_target_checked(&ev));
                        }
                    />
                    <div class="table-cell relative w-11 h-6 bg-gray-200 dark:bg-gray-700 rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-gray-600 after:content-[''] after:absolute after:top-0.5 after:start-[2px] after:bg-cyan-200 after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-gray-400 peer-checked:dark:bg-gray-500"></div>
                    <span class="table-cell text-left ms-3 text-sm font-medium text-gray-900 dark:text-gray-300 select-none">
                        "Toggle Mines"
                    </span>
                </label>
                <label class="table-row cursor-pointer">
                    <input
                        type="checkbox"
                        value=""
                        class="table-cell sr-only peer"
                        checked
                        on:change=move |ev| {
                            set_show_analysis(event_target_checked(&ev));
                        }
                    />
                    <div class="table-cell relative w-11 h-6 bg-gray-200 dark:bg-gray-700 rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-gray-600 after:content-[''] after:absolute after:top-0.5 after:start-[2px] after:bg-cyan-200 after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-gray-400 peer-checked:dark:bg-gray-500"></div>
                    <span class="table-cell text-left ms-3 text-sm font-medium text-gray-900 dark:text-gray-300 select-none">
                        "Toggle Analysis"
                    </span>
                </label>
            </div>
            <div class="w-full max-w-xs flex justify-between items-center">
                <button
                    type="button"
                    class=button_class!(
                        "max-w-xs h-8 select-none rounded-l-md",
                        "bg-neutral-700 hover:bg-neutral-800/90 text-white"
                    )
                    on:click=move |_| prev()
                    disabled=is_beginning
                >
                    "Prev"
                </button>
                <input
                    type="range"
                    min=min
                    max=max
                    value="0"
                    step="1"
                    class="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-cyan-200"
                    node_ref=slider_el
                    on:input=move |_| to_pos()
                    on:change=move |_| to_pos()
                />
                <button
                    type="button"
                    class=button_class!(
                        "max-w-xs h-8 select-none rounded-r-md",
                        "bg-neutral-700 hover:bg-neutral-800/90 text-white"
                    )
                    on:click=move |_| next()
                    disabled=is_end
                >
                    "Next"
                </button>
            </div>
            {move || {
                current_play()
                    .map(move |play| {
                        view! {
                            <div
                                data-hk="1-3-2-8"
                                class="text-xl my-4 text-gray-900 dark:text-gray-200"
                            >
                                "Player "
                                {play.player}
                                ": "
                                {play.action.to_str()}
                                " @ Row: "
                                {play.point.row}
                                ", Col: "
                                {play.point.col}
                            </div>
                        }
                    })
            }}
        </div>
    }
}
