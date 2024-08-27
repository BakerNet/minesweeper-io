use anyhow::Result;
use leptos::*;
use std::{cell::RefCell, rc::Rc};

use crate::components::button_class;
use minesweeper_lib::{
    board::Board,
    cell::{HiddenCell, PlayerCell},
    client::ClientPlayer,
    game::Play,
    replay::{MinesweeperReplay, ReplayPosition, SimplePlayer},
};

#[derive(Clone)]
#[allow(dead_code)]
struct ReplayStore {
    replay: Rc<RefCell<MinesweeperReplay>>,
    cell_read_signals: Vec<Vec<ReadSignal<PlayerCell>>>,
    cell_write_signals: Vec<Vec<WriteSignal<PlayerCell>>>,
    player_write_signals: Vec<WriteSignal<Option<ClientPlayer>>>,
}

impl ReplayStore {
    fn with_current_board(&self, f: impl FnOnce(&Board<PlayerCell>)) {
        let replay: &MinesweeperReplay = &mut (*self.replay).borrow();
        f(replay.current_board())
    }

    fn with_current_players(&self, f: impl FnOnce(&Vec<SimplePlayer>)) {
        let replay: &MinesweeperReplay = &mut (*self.replay).borrow();
        f(replay.current_players())
    }

    fn next(&self) -> Result<ReplayPosition> {
        let replay: &mut MinesweeperReplay = &mut (*self.replay).borrow_mut();
        replay.advance()
    }

    fn prev(&self) -> Result<ReplayPosition> {
        let replay: &mut MinesweeperReplay = &mut (*self.replay).borrow_mut();
        replay.rewind()
    }

    fn flags(&self) -> usize {
        let replay: &mut MinesweeperReplay = &mut (*self.replay).borrow_mut();
        replay.current_flags_and_revealed_mines()
    }

    fn current_play(&self) -> Option<Play> {
        let replay: &mut MinesweeperReplay = &mut (*self.replay).borrow_mut();
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

    let (replay_started, set_replay_started) = create_signal(false);
    let (hide_mines, set_hide_mine) = create_signal(false);
    let (is_beginning, set_beginning) = create_signal(true);
    let (is_end, set_end) = create_signal(false);
    let (current_play, set_current_play) = create_signal::<Option<Play>>(None);

    let replay = ReplayStore {
        replay: Rc::new(RefCell::new(replay)),
        cell_read_signals,
        cell_write_signals,
        player_write_signals,
    };
    let (replay, _) = create_signal(replay);

    let render_current = move || {
        let replay = replay.get_untracked();
        replay.with_current_board(|current_board| {
            current_board
                .rows_iter()
                .enumerate()
                .for_each(|(row, vec)| {
                    vec.iter().enumerate().for_each(|(col, pc)| {
                        let pc = if hide_mines.get_untracked() {
                            match pc {
                                PlayerCell::Hidden(HiddenCell::Mine) => {
                                    PlayerCell::Hidden(HiddenCell::Empty)
                                }
                                PlayerCell::Hidden(HiddenCell::FlagMine) => {
                                    PlayerCell::Hidden(HiddenCell::Flag)
                                }
                                default => *default,
                            }
                        } else {
                            *pc
                        };
                        if replay.cell_read_signals[row][col].get_untracked() != pc {
                            replay.cell_write_signals[row][col](pc);
                        }
                    })
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
    };

    create_effect(move |prev| {
        let hide_mines = hide_mines();
        if replay_started() && prev != Some(hide_mines) {
            render_current();
        }
        hide_mines
    });

    let next = move || {
        let replay = replay.get_untracked();
        let res = replay.next();
        if res.is_ok() {
            render_current();
        }
        if matches!(res, Ok(ReplayPosition::End)) {
            set_end(true);
        }
        set_beginning(false);
    };

    let prev = move || {
        let replay = replay.get_untracked();
        let res = replay.prev();
        if res.is_ok() {
            render_current();
        }
        if matches!(res, Ok(ReplayPosition::Beginning)) {
            set_beginning(true);
        }
        set_end(false);
    };

    view! {
        <div class="flex flex-col items-center space-y-2 mb-8">
            <Show when=move || !replay_started()>
                <button
                    type="button"
                    class=button_class(
                        Some("w-full max-w-xs h-8"),
                        Some("bg-green-700 hover:bg-green-800/90 text-white"),
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
                <button
                    type="button"
                    class=button_class(
                        Some("max-w-xs h-8 select-none"),
                        Some("bg-neutral-700 hover:bg-neutral-800/90 text-white"),
                    )
                    on:click=move |_| {
                        set_hide_mine.update(|b| *b = !*b);
                    }
                >
                    "Toggle Mines"
                </button>
                <div>
                    <button
                        type="button"
                        class=button_class(
                            Some("max-w-xs h-8 select-none rounded-l-md"),
                            Some("bg-neutral-700 hover:bg-neutral-800/90 text-white"),
                        )
                        on:click=move |_| prev()
                        disabled=is_beginning
                    >
                        "Prev"
                    </button>
                    // TODO
                    <span data-hk="1-3-2-8" class="text-xl my-4 text-gray-900 dark:text-gray-200">
                        "Slider Goes Here"
                    </span>
                    <button
                        type="button"
                        class=button_class(
                            Some("max-w-xs h-8 select-none rounded-r-md"),
                            Some("bg-neutral-700 hover:bg-neutral-800/90 text-white"),
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
