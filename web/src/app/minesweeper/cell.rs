use super::{players::player_class, FrontendGame};

use leptos::*;
use minesweeper_lib::{
    cell::{Cell, PlayerCell},
    game::Action as PlayAction,
};
use web_sys::MouseEvent;

#[component]
pub fn ActiveRow(row: usize, cells: Vec<ReadSignal<PlayerCell>>) -> impl IntoView {
    view! {
        <div class="whitespace-nowrap">
            {cells
                .into_iter()
                .enumerate()
                .map(move |(col, cell)| view! { <ActiveCell row=row col=col cell=cell/> })
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
        PlayerCell::Revealed(rc) => match rc.contents {
            Cell::Bomb => String::from("bg-red-600"),
            Cell::Empty(x) => number_class(x as usize),
        },
    }
}

fn cell_player_class(cell: PlayerCell) -> String {
    match cell {
        PlayerCell::Flag => String::from(""),
        PlayerCell::Hidden => String::from(""),
        PlayerCell::Revealed(rc) => player_class(rc.player),
    }
}

pub fn cell_class(content_class: &str, player_class: &str) -> String {
    format!("inline-block text-center border border-solid border-black font-bold align-top h-8 w-8 text-2xl {} {}", content_class, player_class)
}

#[component]
fn ActiveCell(row: usize, col: usize, cell: ReadSignal<PlayerCell>) -> impl IntoView {
    let id = format!("{}_{}", row, col);
    let game = expect_context::<FrontendGame>();
    let (game, _) = create_signal(game);

    let handle_action = move |pa: PlayAction| {
        let res = match pa {
            PlayAction::Reveal => game().try_reveal(row, col),
            PlayAction::Flag => game().try_flag(row, col),
            PlayAction::RevealAdjacent => game().try_reveal_adjacent(row, col),
        };
        res.unwrap_or_else(|e| (game().err_signal)(Some(format!("{:?}", e))));
    };
    let handle_mousedown = move |ev: MouseEvent| {
        let set_skip_signal = { game().set_skip_mouseup };
        if ev.button() == 2 {
            handle_action(PlayAction::Flag);
        }
        if ev.buttons() == 3 {
            set_skip_signal.set(2);
            handle_action(PlayAction::RevealAdjacent);
        }
    };
    let handle_mouseup = move |ev: MouseEvent| {
        leptos_dom::log!("handle_mouseup");
        let (skip_mouseup, set_skip_mouseup) = { (game().skip_mouseup, game().set_skip_mouseup) };
        leptos_dom::log!("{}", skip_mouseup.get());
        if skip_mouseup.get() > 0 {
            set_skip_mouseup.set(skip_mouseup() - 1);
            return;
        }
        if ev.button() == 0 {
            handle_action(PlayAction::Reveal);
        }
    };
    let class = move || {
        let item = cell();
        cell_class(&cell_contents_class(item), &cell_player_class(item))
    };

    view! {
        <span
            class=class
            id=id
            on:mouseup=handle_mouseup
            on:mousedown=handle_mousedown
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
                <img class="object-cover h-full w-full" src="/images/Flag.svg"/>
            </span>
        },
        PlayerCell::Hidden => view! { <span>""</span> },
        PlayerCell::Revealed(rc) => match rc.contents {
            Cell::Bomb => view! {
                <span>
                    <img class="object-cover h-full w-full" src="/images/Mine.svg"/>
                </span>
            },
            Cell::Empty(_) => view! { <span>{format!("{:?}", cell)}</span> },
        },
    }
}
