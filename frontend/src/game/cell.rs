use std::{cell::RefCell, rc::Rc};

use crate::game::FrontendGame;

use leptos::{leptos_dom::console_log, *};
use minesweeper::{
    cell::{Cell, PlayerCell},
    game::Action as PlayAction,
};
use web_sys::MouseEvent;

fn cell_class(cell: PlayerCell) -> String {
    match cell {
        PlayerCell::Flag => String::from("flag"),
        PlayerCell::Hidden => String::from("hidden"),
        PlayerCell::Revealed(rc) => match rc.contents {
            Cell::Bomb => String::from("bomb"),
            Cell::Empty(x) => format!("rev-{}", x),
        },
    }
}

fn player_class(cell: PlayerCell) -> String {
    match cell {
        PlayerCell::Flag => String::from(""),
        PlayerCell::Hidden => String::from(""),
        PlayerCell::Revealed(rc) => match rc.contents {
            Cell::Bomb => format!("p-{}", rc.player),
            Cell::Empty(_) => format!("p-{}", rc.player),
        },
    }
}

#[component]
pub fn Cell(cx: Scope, row: usize, col: usize, cell: ReadSignal<PlayerCell>) -> impl IntoView {
    let id = format!("{}_{}", row, col);

    let handle_action = move |pa: PlayAction| {
        let game = use_context::<Rc<RefCell<FrontendGame>>>(cx).unwrap();
        let game = (*game).borrow();
        let res = match pa {
            PlayAction::Reveal => game.try_reveal(row, col),
            PlayAction::Flag => game.try_flag(row, col),
            PlayAction::RevealAdjacent => game.try_reveal_adjacent(row, col),
        };
        res.unwrap_or_else(|e| (game.err_signal)(Some(format!("{:?}", e))));
    };
    let handle_mousedown = move |ev: MouseEvent| {
        let set_skip_signal = {
            let game = use_context::<Rc<RefCell<FrontendGame>>>(cx).unwrap();
            let game = (*game).borrow();
            game.set_skip_mouseup
        };
        if ev.buttons() == 3 {
            set_skip_signal.set(2);
            handle_action(PlayAction::RevealAdjacent);
        }
    };
    let handle_mouseup = move |ev: MouseEvent| {
        console_log("handle_mouseup");
        let (skip_mouseup, set_skip_mouseup) = {
            let game = use_context::<Rc<RefCell<FrontendGame>>>(cx).unwrap();
            let game = (*game).borrow();
            (game.skip_mouseup, game.set_skip_mouseup)
        };
        console_log(&format!("{}", skip_mouseup.get()));
        if skip_mouseup.get() > 0 {
            set_skip_mouseup.set(skip_mouseup() - 1);
            return;
        }
        if ev.button() == 0 {
            handle_action(PlayAction::Reveal);
        }
        if ev.button() == 2 {
            handle_action(PlayAction::Flag);
        }
    };
    let class = move || {
        let item = cell();
        format!("cell s-30p {} {}", cell_class(item), player_class(item))
    };

    view! { cx,
        <span
        class=class
        id=id
        on:mouseup=handle_mouseup
        on:mousedown=handle_mousedown
        oncontextmenu="event.preventDefault();" >
            {move || {
                let item = cell();
                view! { cx, <CellContents cell=item /> }
            }}
        </span>
    }
}

#[component]
fn CellContents(cx: Scope, cell: PlayerCell) -> impl IntoView {
    match cell {
        PlayerCell::Flag => view! { cx, <span><img src="/images/Flag.svg" /></span>},
        PlayerCell::Hidden => view! { cx, <span>""</span>},
        PlayerCell::Revealed(rc) => match rc.contents {
            Cell::Bomb => view! { cx, <span><img src="/images/Mine.svg" /></span>},
            Cell::Empty(_) => view! { cx, <span>{format!("{:?}", cell)}</span>},
        },
    }
}
