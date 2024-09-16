use anyhow::Result;
use leptos::{html::Input, prelude::*};
use leptos_router::components::*;
use std::sync::{Arc, RwLock};

use crate::button_class;
use minesweeper_lib::{
    board::Board,
    cell::{HiddenCell, PlayerCell},
    client::ClientPlayer,
    game::Play,
    replay::{MinesweeperReplay, ReplayPosition, Replayable, SimplePlayer},
};

#[derive(Clone)]
#[allow(dead_code)]
struct ReplayStore {
    replay: Arc<RwLock<MinesweeperReplay>>,
    cell_read_signals: Arc<Vec<Vec<ReadSignal<PlayerCell>>>>,
    cell_write_signals: Arc<Vec<Vec<WriteSignal<PlayerCell>>>>,
    player_write_signals: Arc<Vec<WriteSignal<Option<ClientPlayer>>>>,
}

impl ReplayStore {
    fn with_current_board(&self, f: impl FnOnce(&Board<PlayerCell>)) {
        let replay: &MinesweeperReplay = &mut (*self.replay).read().unwrap();
        f(replay.current_board())
    }

    fn with_current_players(&self, f: impl FnOnce(&Vec<SimplePlayer>)) {
        let replay: &MinesweeperReplay = &mut (*self.replay).read().unwrap();
        f(replay.current_players())
    }

    fn next(&self) -> Result<ReplayPosition> {
        let replay: &mut MinesweeperReplay = &mut (*self.replay).write().unwrap();
        replay.advance()
    }

    fn prev(&self) -> Result<ReplayPosition> {
        let replay: &mut MinesweeperReplay = &mut (*self.replay).write().unwrap();
        replay.rewind()
    }

    fn to_pos(&self, pos: usize) -> Result<ReplayPosition> {
        let replay: &mut MinesweeperReplay = &mut (*self.replay).write().unwrap();
        let pos = ReplayPosition::from_pos(pos, replay.len());
        replay.to_pos(pos)
    }

    fn flags(&self) -> usize {
        let replay: &mut MinesweeperReplay = &mut (*self.replay).write().unwrap();
        replay.current_flags_and_revealed_mines()
    }

    fn current_play(&self) -> Option<Play> {
        let replay: &mut MinesweeperReplay = &mut (*self.replay).write().unwrap();
        replay.current_play()
    }
}

#[component]
pub fn ReplayControls(
    replay: MinesweeperReplay,
    cell_read_signals: Vec<Vec<ReadSignal<PlayerCell>>>,
    cell_write_signals: Vec<Vec<WriteSignal<PlayerCell>>>,
    set_flag_count: WriteSignal<usize>,
    player_write_signals: Vec<WriteSignal<Option<ClientPlayer>>>,
) -> impl IntoView {
    log::debug!("replay log length: {}", replay.len());
    let min = 0;
    let max = replay.len() - 1;
    let slider_el = NodeRef::<Input>::new();

    let (replay_started, set_replay_started) = signal(false);
    let (hide_mines, set_hide_mines) = signal(false);
    let (is_beginning, set_beginning) = signal(true);
    let (is_end, set_end) = signal(false);
    let (current_play, set_current_play) = signal::<Option<Play>>(None);

    let replay = ReplayStore {
        replay: Arc::new(RwLock::new(replay)),
        cell_read_signals: cell_read_signals.into(),
        cell_write_signals: cell_write_signals.into(),
        player_write_signals: player_write_signals.into(),
    };
    let replay = StoredValue::new(replay);

    let render_cell = move |replay: &ReplayStore, row: usize, col: usize, pc: &PlayerCell| {
        let pc = if hide_mines.get_untracked() {
            match pc {
                PlayerCell::Hidden(HiddenCell::Mine) => PlayerCell::Hidden(HiddenCell::Empty),
                PlayerCell::Hidden(HiddenCell::FlagMine) => PlayerCell::Hidden(HiddenCell::Flag),
                default => *default,
            }
        } else {
            *pc
        };
        if replay.cell_read_signals[row][col].get_untracked() != pc {
            replay.cell_write_signals[row][col](pc);
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
            replay.with_current_players(|current_players| {
                current_players.iter().enumerate().for_each(|(i, p)| {
                    replay.player_write_signals[i].update(|cp| {
                        if let Some(cp) = cp.as_mut() {
                            p.update_client_player(cp);
                        }
                    });
                })
            });
            set_flag_count(replay.flags());
            set_current_play(replay.current_play());
        })
    };

    Effect::watch(
        hide_mines,
        move |hide_mines, _, prev| {
            if replay_started.get_untracked() && prev != Some(*hide_mines) {
                render_current();
            }
            *hide_mines
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
                slider.set_value(&format!("{}", new_pos));
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
                slider.set_value(&format!("{}", new_pos));
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
            log::debug!("Slider: {} / {}", pos, max);
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

    view! {
        <div class="flex flex-col items-center space-y-2 mb-8">
            <Show when=move || !replay_started()>
                <button
                    type="button"
                    class=button_class!(
                        "max-w-xs h-10 rounded-lg text-lg",
                        "bg-green-700 hover:bg-green-800/90 text-white"
                    )
                    on:click=move |_| {
                        set_replay_started(true);
                        render_current();
                    }
                >
                    "Start Replay"
                </button>
            </Show>
            <Show when=replay_started>
                <label class="inline-flex items-center cursor-pointer">
                    <input
                        type="checkbox"
                        value=""
                        class="sr-only peer"
                        checked
                        on:change=move |ev| {
                            set_hide_mines(!event_target_checked(&ev));
                        }
                    />
                    <div class="relative w-11 h-6 bg-gray-200 dark:bg-gray-700 rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-gray-600 after:content-[''] after:absolute after:top-0.5 after:start-[2px] after:bg-cyan-200 after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all dark:border-gray-600 peer-checked:bg-gray-400 peer-checked:dark:bg-gray-500"></div>
                    <span class="ms-3 text-sm font-medium text-gray-900 dark:text-gray-300">
                        "Toggle Mines"
                    </span>
                </label>
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
            </Show>
        </div>
    }
}

#[component]
pub fn OpenReplay() -> impl IntoView {
    view! {
        <div class="flex flex-col items-center space-y-4 mb-8">
            <A
                href="replay"
                attr:class=button_class!(
                    "w-full max-w-xs h-8",
                    "bg-neutral-700 hover:bg-neutral-800/90 text-white"
                )
            >
                "Open Replay"
            </A>
        </div>
    }
}
