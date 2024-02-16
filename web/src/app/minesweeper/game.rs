use leptos::*;
use leptos_use::{core::ConnectionReadyState, use_websocket, UseWebsocketReturn};
use minesweeper_lib::game::Action as PlayAction;
use std::rc::Rc;

use minesweeper_lib::cell::PlayerCell;

use crate::app::minesweeper::client::PlayersContext;

use super::cell::{ActiveRow, InactiveRow};
use super::client::FrontendGame;
use super::players::{ActivePlayers, InactivePlayers};
use super::GameInfo;

#[component]
fn GameBorder(children: Children) -> impl IntoView {
    view! {
        <div class="select-none overflow-x-auto overflow-y-hidden mb-12">
            <div class="w-fit border-solid border border-black mx-auto">
                <div class="w-fit border-groove border-24">{children()}</div>
            </div>
        </div>
    }
}

#[component]
pub fn ActiveGame<F>(game_info: GameInfo, refetch: F) -> impl IntoView
where
    F: Fn() + Clone + 'static,
{
    let (error, set_error) = create_signal::<Option<String>>(None);

    let game_id = game_info.game_id.clone();
    let UseWebsocketReturn {
        ready_state,
        message,
        send,
        close,
        ..
    } = use_websocket(&format!("/api/websocket/game/{}", &game_id));

    let (game, read_signals) = FrontendGame::new(
        game_info.clone(),
        set_error,
        Rc::new(send.clone()),
        Rc::new(close.clone()),
    );
    let (game_signal, _) = create_signal(game.clone());

    provide_context::<PlayersContext>(PlayersContext::from(&game));

    create_effect(move |_| {
        log::debug!("before ready_state");
        let state = ready_state();
        if state == ConnectionReadyState::Open {
            log::debug!("ready_state Open");
            game_signal().send(&game_id);
        } else if state == ConnectionReadyState::Closed {
            log::debug!("ready_state Closed");
            refetch();
        }
    });

    create_effect(move |_| {
        log::debug!("before message");
        if let Some(msg) = message() {
            log::debug!("after message {}", msg);
            let res = game_signal().handle_message(&msg);
            if let Err(e) = res {
                (game_signal().err_signal)(Some(format!("{:?}", e)))
            } else {
                (game_signal().err_signal)(None)
            }
        }
    });

    create_effect(move |last| {
        game_signal().join_trigger.track();
        if last.is_some() {
            game_signal().send("Play");
        }
    });

    let handle_action = move |pa: PlayAction, row: usize, col: usize| {
        let res = match pa {
            PlayAction::Reveal => game_signal().try_reveal(row, col),
            PlayAction::Flag => game_signal().try_flag(row, col),
            PlayAction::RevealAdjacent => game_signal().try_reveal_adjacent(row, col),
        };
        res.unwrap_or_else(|e| (game_signal().err_signal)(Some(format!("{:?}", e))));
    };
    // TODO - game lifecycle UI (started indicators, ended indicators, countdown / starting alerts, etc.)

    view! {
        <div class="text-center">
            <ActivePlayers/>
            <GameBorder>
                {read_signals
                    .into_iter()
                    .enumerate()
                    .map(move |(row, vec)| {
                        view! {
                            <ActiveRow
                                row=row
                                cells=vec
                                skip_mouseup=game.skip_mouseup
                                set_skip_mouseup=game.set_skip_mouseup
                                handle_action
                            />
                        }
                    })
                    .collect_view()}
            </GameBorder>
            <div class="text-red-600 h-8">{error}</div>
        </div>
    }
}

#[component]
pub fn InactiveGame(game_info: GameInfo) -> impl IntoView {
    let players = game_info.players.clone();
    let board = match game_info.final_board {
        None => vec![vec![PlayerCell::Hidden; game_info.cols]; game_info.rows],
        Some(b) => b,
    };

    view! {
        <div class="text-center">
            <InactivePlayers players/>
            <GameBorder>
                {board
                    .into_iter()
                    .enumerate()
                    .map(move |(row, vec)| view! { <InactiveRow row=row cells=vec/> })
                    .collect_view()}
            </GameBorder>
        </div>
    }
}
