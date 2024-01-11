use leptos::*;
use leptos_router::*;
use leptos_use::{core::ConnectionReadyState, use_websocket, UseWebsocketReturn};
use std::rc::Rc;

use minesweeper_lib::cell::PlayerCell;

use super::cell::{ActiveRow, InactiveRow};
use super::client::FrontendGame;
use super::GameInfo;

// TODO - get active games working
#[component]
pub fn ActiveGame(game_info: GameInfo) -> impl IntoView {
    let (error, set_error) = create_signal::<Option<String>>(None);

    let UseWebsocketReturn {
        ready_state,
        message,
        send,
        close,
        ..
    } = use_websocket(&format!("/api/websocket/game/{}", &game_info.game_id));

    let (game, read_signals) = FrontendGame::new(
        game_info.clone(),
        set_error,
        Rc::new(send.clone()),
        Rc::new(close.clone()),
    );

    provide_context::<FrontendGame>(game.clone());

    let game_clone = game.clone();
    create_effect(move |_| {
        log::debug!("before ready_state");
        if ready_state() == ConnectionReadyState::Open {
            log::debug!("after ready_state");
            game_clone.send(&game_info.game_id);
        }
    });

    let game_clone = game.clone();
    create_effect(move |_| {
        log::debug!("before message");
        if let Some(msg) = message() {
            log::debug!("after message {}", msg);
            let res = game_clone.handle_message(&msg);
            if let Err(e) = res {
                (game_clone.err_signal)(Some(format!("{:?}", e)))
            } else {
                (game_clone.err_signal)(None)
            }
        }
    });

    view! {
        <div class="Game">
            <Outlet/>
            <div class="error" style:height="24px">
                {error}
            </div>
            <div class="board">
                {read_signals
                    .into_iter()
                    .enumerate()
                    .map(move |(row, vec)| view! { <ActiveRow row=row cells=vec/> })
                    .collect_view()}
            </div>
        </div>
    }
}

#[component]
pub fn InactiveGame(game_info: GameInfo) -> impl IntoView {
    let board = match game_info.final_board {
        None => vec![vec![PlayerCell::Hidden; game_info.cols]; game_info.rows],
        Some(b) => b,
    };

    view! {
        <div class="Game">
            <Outlet/>
            <div class="board">
                {board
                    .into_iter()
                    .enumerate()
                    .map(move |(row, vec)| view! { <InactiveRow row=row cells=vec/> })
                    .collect_view()}
            </div>
        </div>
    }
}
