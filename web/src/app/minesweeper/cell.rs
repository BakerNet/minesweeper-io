use crate::components::icons::{Flag, Mine};

use super::players::player_class;

use leptos::*;
use minesweeper_lib::{
    board::BoardPoint,
    cell::{Cell, PlayerCell},
};
use web_sys::MouseEvent;

#[component]
pub fn ActiveRow<F, F2>(
    row: usize,
    cells: Vec<ReadSignal<PlayerCell>>,
    set_active_cell: WriteSignal<BoardPoint>,
    mousedown_handler: F,
    mouseup_handler: F2,
) -> impl IntoView
where
    F: Fn(MouseEvent, usize, usize) + Copy + 'static,
    F2: Fn(MouseEvent, usize, usize) + Copy + 'static,
{
    view! {
        <div class="whitespace-nowrap">
            {cells
                .into_iter()
                .enumerate()
                .map(move |(col, cell)| {
                    view! {
                        <ActiveCell
                            row=row
                            col=col
                            cell=cell
                            set_active=set_active_cell
                            mousedown_handler
                            mouseup_handler
                        />
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
pub fn InactiveRow(row: usize, cells: Vec<PlayerCell>) -> impl IntoView {
    view! {
        <div class="whitespace-nowrap">
            {cells
                .into_iter()
                .enumerate()
                .map(move |(col, cell)| view! { <InactiveCell row=row col=col cell=cell/> })
                .collect_view()}
        </div>
    }
}

pub fn number_class(num: usize) -> String {
    String::from(match num {
        1 => "text-blue-600",
        2 => "text-green-600",
        3 => "text-red-600",
        4 => "text-blue-950",
        5 => "text-rose-900",
        6 => "text-teal-600",
        7 => "text-neutral-950",
        8 => "text-neutral-600",
        _ => "",
    })
}

fn cell_contents_class(cell: PlayerCell) -> String {
    match cell {
        PlayerCell::Flag => String::from("bg-neutral-500"),
        PlayerCell::Hidden => String::from("bg-neutral-500"),
        PlayerCell::HiddenMine => String::from("bg-neutral-500"),
        PlayerCell::Revealed(rc) => match rc.contents {
            Cell::Mine => String::from("bg-red-600"),
            Cell::Empty(x) => number_class(x as usize),
        },
    }
}

fn cell_player_class(cell: PlayerCell) -> String {
    match cell {
        PlayerCell::Revealed(rc) if matches!(rc.contents, Cell::Empty(_)) => {
            player_class(rc.player)
        }
        _ => String::from(""),
    }
}

pub fn cell_class(content_class: &str, player_class: &str) -> String {
    format!("inline-block text-center border border-solid border-black font-bold align-top h-8 w-8 text-2xl {} {}", content_class, player_class)
}

#[component]
fn ActiveCell<F, F2>(
    row: usize,
    col: usize,
    cell: ReadSignal<PlayerCell>,
    set_active: WriteSignal<BoardPoint>,
    mousedown_handler: F,
    mouseup_handler: F2,
) -> impl IntoView
where
    F: Fn(MouseEvent, usize, usize) + Copy + 'static,
    F2: Fn(MouseEvent, usize, usize) + Copy + 'static,
{
    let id = format!("{}_{}", row, col);
    let class = move || {
        let item = cell();
        cell_class(&cell_contents_class(item), &cell_player_class(item))
    };

    view! {
        <span
            class=class
            id=id
            on:mousedown=move |ev| mousedown_handler(ev, row, col)
            on:mouseup=move |ev| mouseup_handler(ev, row, col)
            on:mouseenter=move |_| set_active(BoardPoint { row, col })
            oncontextmenu="event.preventDefault();"
        >
            {move || {
                let item = cell();
                view! { <CellContents cell=item/> }
            }}

        </span>
    }
}

#[component]
fn InactiveCell(row: usize, col: usize, cell: PlayerCell) -> impl IntoView {
    let id = format!("{}_{}", row, col);
    let class = cell_class(&cell_contents_class(cell), &cell_player_class(cell));

    view! {
        <span class=class id=id oncontextmenu="event.preventDefault();">
            <CellContents cell/>
        </span>
    }
}

#[component]
fn CellContents(cell: PlayerCell) -> impl IntoView {
    match cell {
        PlayerCell::Flag => view! {
            <span>
                <Flag/>
            </span>
        },
        PlayerCell::Hidden => view! { <span>""</span> },
        PlayerCell::HiddenMine => view! {
            <span>
                <Mine/>
            </span>
        },
        PlayerCell::Revealed(rc) => match rc.contents {
            Cell::Mine => view! {
                <span>
                    <Mine/>
                </span>
            },
            Cell::Empty(_) => view! { <span>{format!("{:?}", cell)}</span> },
        },
    }
}
