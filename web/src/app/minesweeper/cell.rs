use leptos::*;
use web_sys::MouseEvent;

use minesweeper_lib::{
    board::BoardPoint,
    cell::{Cell, HiddenCell, PlayerCell},
};

use crate::components::{
    cell_class,
    icons::{Flag, Mine},
    number_class, player_class,
};

fn cell_contents_class(cell: PlayerCell) -> String {
    match cell {
        PlayerCell::Hidden(_) => String::from("bg-neutral-500"),
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

#[component]
pub fn ActiveCell<F, F2>(
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
                view! { <CellContents cell=item /> }
            }}

        </span>
    }
}

#[component]
pub fn InactiveCell(row: usize, col: usize, cell: PlayerCell) -> impl IntoView {
    let id = format!("{}_{}", row, col);
    let class = cell_class(&cell_contents_class(cell), &cell_player_class(cell));

    view! {
        <span class=class id=id oncontextmenu="event.preventDefault();">
            <CellContents cell />
        </span>
    }
}

#[component]
fn CellContents(cell: PlayerCell) -> impl IntoView {
    match cell {
        PlayerCell::Hidden(hc) => match hc {
            HiddenCell::Empty => view! { <span>""</span> },
            HiddenCell::Flag => view! {
                <span>
                    <Flag />
                </span>
            },
            HiddenCell::Mine => view! {
                <span>
                    <Mine />
                </span>
            },
            HiddenCell::FlagMine => view! {
                <span>
                    <Flag />
                </span>
            },
        },
        PlayerCell::Revealed(rc) => match rc.contents {
            Cell::Mine => view! {
                <span>
                    <Mine />
                </span>
            },
            Cell::Empty(_) => view! { <span>{format!("{:?}", cell)}</span> },
        },
    }
}
