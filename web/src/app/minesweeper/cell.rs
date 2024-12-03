use leptos::either::*;
use leptos::prelude::*;
use minesweeper_lib::{analysis::AnalyzedCell, replay::ReplayAnalysisCell};
use web_sys::{MouseEvent, TouchEvent};

use minesweeper_lib::{
    board::BoardPoint,
    cell::{Cell, HiddenCell, PlayerCell},
};

use crate::{
    cell_class,
    components::icons::{Flag, FlagContrast, Mine},
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
pub fn ActiveCell<F, F2, F3, F4>(
    row: usize,
    col: usize,
    cell: ReadSignal<PlayerCell>,
    set_active: WriteSignal<BoardPoint>,
    mousedown_handler: F,
    mouseup_handler: F2,
    touchstart_handler: F3,
    touchend_handler: F4,
) -> impl IntoView
where
    F: Fn(MouseEvent, usize, usize) + Copy + 'static,
    F2: Fn(MouseEvent, usize, usize) + Copy + 'static,
    F3: Fn(TouchEvent, usize, usize) + Copy + 'static,
    F4: Fn(TouchEvent, usize, usize) + Copy + 'static,
{
    let id = format!("{}_{}", row, col);
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
    let id = format!("{}_{}", row, col);
    let class = cell_class!(cell_contents_class(cell, false), cell_player_class(cell));

    view! {
        <span class=class id=id oncontextmenu="event.preventDefault();">
            <CellContents cell />
        </span>
    }
}

#[component]
pub fn ReplayCell(row: usize, col: usize, cell: ReadSignal<ReplayAnalysisCell>) -> impl IntoView {
    let id = format!("{}_{}", row, col);
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
