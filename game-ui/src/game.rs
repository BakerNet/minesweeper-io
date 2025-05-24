use leptos::{either::*, ev, prelude::*};
use leptos_use::{use_document, use_event_listener};
use std::time::Duration;
use web_sys::{KeyboardEvent, MouseEvent, TouchEvent};

use minesweeper_lib::{
    analysis::AnalyzedCell,
    board::{Board, BoardPoint},
    cell::{Cell, HiddenCell, PlayerCell},
    game::Action as PlayAction,
    replay::ReplayAnalysisCell,
};

use crate::{
    cell_class,
    icons::{Flag, FlagContrast, Mine},
    number_class, player_class,
};

fn cell_contents_class(cell: PlayerCell, active: bool) -> &'static str {
    match cell {
        PlayerCell::Hidden(HiddenCell::Flag) if !active => "bg-red-400/40",
        PlayerCell::Hidden(_) => "bg-neutral-500",
        PlayerCell::Revealed(rc) => match rc.contents {
            Cell::Mine => "bg-red-600",
            Cell::Empty(x) => number_class!(x),
        },
    }
}

fn cell_replay_class(cell: PlayerCell, analysis: Option<AnalyzedCell>) -> &'static str {
    match cell {
        PlayerCell::Hidden(HiddenCell::Flag) if matches!(analysis, Some(AnalyzedCell::Empty)) => {
            "bg-red-400/40"
        }
        PlayerCell::Hidden(HiddenCell::Empty) if matches!(analysis, Some(AnalyzedCell::Empty)) => {
            "bg-green-400/40"
        }
        PlayerCell::Hidden(_) if matches!(analysis, Some(AnalyzedCell::Mine)) => "bg-yellow-400/40",
        PlayerCell::Hidden(_) => "bg-neutral-500",
        PlayerCell::Revealed(rc) => match rc.contents {
            Cell::Mine => "bg-red-600",
            Cell::Empty(x) => number_class!(x),
        },
    }
}

fn cell_player_class(cell: PlayerCell) -> &'static str {
    match cell {
        PlayerCell::Revealed(rc) if matches!(rc.contents, Cell::Empty(_)) => {
            player_class!(rc.player)
        }
        _ => "",
    }
}

#[component]
pub fn ActiveCell<F, F2, F3, F4, F5>(
    row: usize,
    col: usize,
    cell: ReadSignal<PlayerCell>,
    set_active: F,
    mousedown_handler: F2,
    mouseup_handler: F3,
    touchstart_handler: F4,
    touchend_handler: F5,
) -> impl IntoView
where
    F: Fn(BoardPoint) + Copy + 'static,
    F2: Fn(MouseEvent, usize, usize) + Copy + 'static,
    F3: Fn(MouseEvent, usize, usize) + Copy + 'static,
    F4: Fn(TouchEvent, usize, usize) + Copy + 'static,
    F5: Fn(TouchEvent, usize, usize) + Copy + 'static,
{
    let id = format!("{row}_{col}");
    let class = move || {
        let item = cell();
        cell_class!(cell_contents_class(item, true), cell_player_class(item))
    };

    view! {
        <span
            class=class
            id=id
            on:mousedown=move |ev| mousedown_handler(ev, row, col)
            on:mouseup=move |ev| mouseup_handler(ev, row, col)
            on:touchstart=move |ev| touchstart_handler(ev, row, col)
            on:touchend=move |ev| touchend_handler(ev, row, col)
            on:touchcancel=move |ev| touchend_handler(ev, row, col)
            on:mouseenter=move |_| set_active(BoardPoint { row, col })
            oncontextmenu="event.preventDefault();"
        >
            {move || {
                let item = cell();
                view! { <CellContents cell=item /> }
            }}

        </span>
    }
}

#[component]
pub fn InactiveCell(row: usize, col: usize, cell: PlayerCell) -> impl IntoView {
    let id = format!("{row}_{col}");
    let class = cell_class!(cell_contents_class(cell, false), cell_player_class(cell));

    view! {
        <span class=class id=id oncontextmenu="event.preventDefault();">
            <CellContents cell />
        </span>
    }
}

#[component]
pub fn ReplayCell(row: usize, col: usize, cell: ReadSignal<ReplayAnalysisCell>) -> impl IntoView {
    let id = format!("{row}_{col}");
    let class = move || {
        let ReplayAnalysisCell(item, analysis) = cell();
        cell_class!(cell_replay_class(item, analysis), cell_player_class(item))
    };

    view! {
        <span class=class id=id oncontextmenu="event.preventDefault();">
            {move || {
                let ReplayAnalysisCell(item, _) = cell();
                view! { <CellContents cell=item /> }
            }}
        </span>
    }
}

#[component]
fn CellContents(cell: PlayerCell) -> impl IntoView {
    match cell {
        PlayerCell::Hidden(hc) => match hc {
            HiddenCell::Empty => EitherOf7::A(view! { <span>""</span> }),
            HiddenCell::Flag => EitherOf7::B(view! {
                <span class="flag">
                    <Flag />
                </span>
            }),
            HiddenCell::Mine => EitherOf7::C(view! {
                <span>
                    <Mine />
                </span>
            }),
            HiddenCell::FlagMine => EitherOf7::D(view! {
                <span class="block w-full h-full relative">
                    <span class="inline-block h-6 w-6 bottom-0 left-0 absolute">
                        <Mine />
                    </span>
                    <span class="inline-block h-6 w-6 top-0 right-0 absolute">
                        <FlagContrast />
                    </span>
                </span>
            }),
        },
        PlayerCell::Revealed(rc) => match rc.contents {
            Cell::Mine => EitherOf7::E(view! {
                <span>
                    <Mine />
                </span>
            }),
            Cell::Empty(0) => EitherOf7::F(view! { <span></span> }),
            Cell::Empty(n) => EitherOf7::G(view! { <span>{n}</span> }),
        },
    }
}

#[component]
fn GameBorder<F>(set_active: F, children: Children) -> impl IntoView
where
    F: Fn(bool) + Copy + 'static,
{
    view! {
        <div class="select-none overflow-x-auto overflow-y-hidden mb-8">
            <div class="w-fit border-solid border border-black mx-auto">
                <div
                    class="w-fit border-groove border-24 bg-gray-900"
                    on:mouseenter=move |_| set_active(true)
                    on:mouseleave=move |_| set_active(false)
                >
                    {children()}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ActiveGame<F>(
    action_handler: F,
    cell_read_signals: impl IntoIterator<Item = Vec<ReadSignal<PlayerCell>>> + 'static,
) -> impl IntoView
where
    F: Fn(PlayAction, usize, usize) + Copy + 'static,
{
    let skip_mouseup = StoredValue::<usize>::new(0);
    let game_is_active = StoredValue::new(false);
    let active_cell = StoredValue::new(BoardPoint { row: 0, col: 0 });
    let touch_timer = StoredValue::new(None::<TimeoutHandle>);

    let set_active_cell = move |bp: BoardPoint| {
        active_cell.set_value(bp);
    };
    let set_game_is_active = move |active: bool| {
        game_is_active.set_value(active);
    };
    let set_touch_timer = move |to: Option<TimeoutHandle>| {
        touch_timer.set_value(to);
    };

    let handle_keydown = move |ev: KeyboardEvent| {
        if !game_is_active.get_value() {
            return;
        }
        let BoardPoint { row, col } = active_cell.get_value();
        match ev.key().as_str() {
            " " => {
                ev.prevent_default();
                action_handler(PlayAction::Reveal, row, col);
            }
            "d" => {
                action_handler(PlayAction::RevealAdjacent, row, col);
            }
            "f" => {
                action_handler(PlayAction::Flag, row, col);
            }
            _ => {}
        }
    };
    let _ = use_event_listener(use_document(), ev::keydown, handle_keydown);

    let handle_mousedown = move |ev: MouseEvent, row: usize, col: usize| {
        if ev.button() == 2 {
            action_handler(PlayAction::Flag, row, col);
        }
        if ev.buttons() == 3 {
            skip_mouseup.set_value(2);
            action_handler(PlayAction::RevealAdjacent, row, col);
        }
    };
    let handle_mouseup = move |ev: MouseEvent, row: usize, col: usize| {
        if skip_mouseup.get_value() > 0 {
            skip_mouseup.update_value(|x| *x -= 1);
            return;
        }
        if ev.button() == 0 {
            action_handler(PlayAction::Reveal, row, col);
        }
    };

    let handle_touchstart = move |_: TouchEvent, row: usize, col: usize| {
        let res = set_timeout_with_handle(
            move || {
                action_handler(PlayAction::Flag, row, col);
                set_touch_timer(None);
            },
            Duration::from_millis(200),
        );
        if let Ok(t) = res {
            set_touch_timer(Some(t));
        }
    };

    let handle_touchend = move |_: TouchEvent, _: usize, _: usize| {
        let timer = touch_timer.get_value();
        if let Some(t) = timer {
            t.clear();
            set_touch_timer(None);
        }
    };

    let active_cell = move |row: usize, col: usize, cell: ReadSignal<PlayerCell>| {
        view! {
            <ActiveCell
                row=row
                col=col
                cell=cell
                set_active=set_active_cell
                mousedown_handler=handle_mousedown
                mouseup_handler=handle_mouseup
                touchstart_handler=handle_touchstart
                touchend_handler=handle_touchend
            />
        }
    };
    let cell_row = move |row: usize, vec: &[ReadSignal<PlayerCell>]| {
        view! {
            <div class="whitespace-nowrap">
                {vec
                    .iter()
                    .copied()
                    .enumerate()
                    .map(move |(col, cell)| { active_cell(row, col, cell) })
                    .collect_view()}
            </div>
        }
    };
    let cells = cell_read_signals
        .into_iter()
        .enumerate()
        .map(|(i, v)| cell_row(i, v.as_ref()))
        .collect_view();

    view! { <GameBorder set_active=set_game_is_active>{cells}</GameBorder> }
}

#[component]
pub fn InactiveGame(board: Board<PlayerCell>) -> impl IntoView {
    let cell_row = |(row, vec): (usize, &[PlayerCell])| {
        view! {
            <div class="whitespace-nowrap">
                {vec
                    .iter()
                    .copied()
                    .enumerate()
                    .map(move |(col, cell)| {
                        view! { <InactiveCell row=row col=col cell=cell /> }
                    })
                    .collect_view()}
            </div>
        }
    };
    let cells = view! { {board.rows_iter().enumerate().map(cell_row).collect_view()} };

    view! { <GameBorder set_active=move |_| {}>{cells}</GameBorder> }
}

#[component]
pub fn ReplayGame(
    cell_read_signals: impl IntoIterator<Item = Vec<ReadSignal<ReplayAnalysisCell>>> + 'static,
) -> impl IntoView {
    let cell_row = |row: usize, cells: &[ReadSignal<ReplayAnalysisCell>]| {
        view! {
            <div class="whitespace-nowrap">
                {cells
                    .iter()
                    .enumerate()
                    .map(move |(col, &cell)| view! { <ReplayCell row=row col=col cell=cell /> })
                    .collect_view()}
            </div>
        }
    };

    let cells = cell_read_signals
        .into_iter()
        .enumerate()
        .map(|(i, v)| cell_row(i, v.as_ref()))
        .collect_view();

    view! { <GameBorder set_active=move |_| ()>{cells}</GameBorder> }
}
