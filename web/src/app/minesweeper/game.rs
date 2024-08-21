use codee::string::FromToStringCodec;
use leptos::ev::keydown;
use leptos::*;
use leptos_router::use_params_map;
use leptos_use::{core::ConnectionReadyState, use_websocket, UseWebSocketReturn};
use leptos_use::{use_document, use_event_listener};
use std::rc::Rc;
use web_sys::{KeyboardEvent, MouseEvent};

use minesweeper_lib::{board::BoardPoint, cell::PlayerCell, game::Action as PlayAction};

use super::{
    cell::{ActiveRow, InactiveRow},
    client::{FrontendGame, PlayersContext},
    entry::ReCreateGame,
    players::{ActivePlayers, InactivePlayers},
    {GameInfo, GameSettings},
};

#[cfg(feature = "ssr")]
use crate::backend::{AuthSession, GameManager};
#[cfg(feature = "ssr")]
use minesweeper_lib::client::ClientPlayer;

#[server(GetGame, "/api")]
pub async fn get_game(game_id: String) -> Result<GameInfo, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    let game = game_manager
        .get_game(&game_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let is_owner = if let Some(user) = auth_session.user {
        match game.owner {
            None => false,
            Some(owner) => user.id == owner,
        }
    } else {
        false
    };
    let players = game_manager
        .get_players(&game_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let players = players.into_iter().map(ClientPlayer::from).fold(
        vec![None; game.max_players as usize],
        |mut acc, p| {
            acc[p.player_id] = Some(p.clone());
            acc
        },
    );
    Ok(GameInfo {
        game_id: game.game_id,
        has_owner: game.owner.is_some(),
        is_owner,
        rows: game.rows as usize,
        cols: game.cols as usize,
        num_mines: game.num_mines as usize,
        max_players: game.max_players,
        is_started: game.is_started,
        is_completed: game.is_completed,
        final_board: game.final_board,
        players,
    })
}

#[component]
pub fn Game() -> impl IntoView {
    let params = use_params_map();
    let game_id = move || params.get().get("id").cloned().unwrap_or_default();
    let game_info = create_resource(game_id, get_game);
    let refetch = move || game_info.refetch();

    provide_context::<Resource<String, Result<GameInfo, ServerFnError>>>(game_info);

    let game_view = move |game_info: GameInfo| match game_info.is_completed {
        true => view! { <InactiveGame game_info/> },
        false => view! { <ActiveGame game_info refetch/> },
    };

    view! {
        <Transition fallback=move || {
            view! { <div>"Loading..."</div> }
        }>
            {move || {
                game_info
                    .get()
                    .map(|game_info| {
                        view! {
                            <ErrorBoundary fallback=|_| {
                                view! { <div class="text-red-600">"Game not found"</div> }
                            }>{move || { game_info.clone().map(game_view) }}</ErrorBoundary>
                        }
                    })
            }}

        </Transition>
    }
}

#[component]
fn GameBorder<F>(set_active: F, children: Children) -> impl IntoView
where
    F: Fn(bool) + Copy + 'static,
{
    view! {
        <div class="select-none overflow-x-auto overflow-y-hidden mb-12">
            <div class="w-fit border-solid border border-black mx-auto">
                <div
                    class="w-fit border-groove border-24"
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
pub fn ActiveGame<F>(game_info: GameInfo, refetch: F) -> impl IntoView
where
    F: Fn() + Clone + 'static,
{
    let (error, set_error) = create_signal::<Option<String>>(None);

    let game_id = game_info.game_id.clone();
    let UseWebSocketReturn {
        ready_state,
        message,
        send,
        ..
    } = use_websocket::<String, FromToStringCodec>(&format!("/api/websocket/game/{}", &game_id));

    let (game, read_signals) =
        FrontendGame::new(game_info.clone(), set_error, Rc::new(send.clone()));
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

    create_effect(move |last: Option<bool>| {
        game_signal().join_trigger.track();
        log::debug!("join_trigger rec: {last:?}");
        if let Some(sent) = last {
            if !sent {
                game_signal().send(&String::from("Play"));
                return true;
            }
        }
        false
    });

    let (skip_mouseup, set_skip_mouseup) = create_signal::<usize>(0);
    let (game_is_active, set_game_is_active) = create_signal(false);
    let (active_cell, set_active_cell) = create_signal(BoardPoint { row: 0, col: 0 });

    let handle_action = move |pa: PlayAction, row: usize, col: usize| {
        let res = match pa {
            PlayAction::Reveal => game_signal.get().try_reveal(row, col),
            PlayAction::Flag => game_signal.get().try_flag(row, col),
            PlayAction::RevealAdjacent => game_signal.get().try_reveal_adjacent(row, col),
        };
        res.unwrap_or_else(|e| (game_signal.get().err_signal)(Some(format!("{:?}", e))));
    };

    let handle_keydown = move |ev: KeyboardEvent| {
        if !game_is_active.get() {
            return;
        }
        let BoardPoint { row, col } = active_cell.get();
        match ev.key().as_str() {
            " " => {
                ev.prevent_default();
                handle_action(PlayAction::Reveal, row, col);
            }
            "d" => {
                handle_action(PlayAction::RevealAdjacent, row, col);
            }
            "f" => {
                handle_action(PlayAction::Flag, row, col);
            }
            _ => {}
        }
    };
    let _ = use_event_listener(use_document(), keydown, handle_keydown);

    let handle_mousedown = move |ev: MouseEvent, row: usize, col: usize| {
        let set_skip_signal = { set_skip_mouseup };
        if ev.button() == 2 {
            handle_action(PlayAction::Flag, row, col);
        }
        if ev.buttons() == 3 {
            set_skip_signal.set(2);
            handle_action(PlayAction::RevealAdjacent, row, col);
        }
    };
    let handle_mouseup = move |ev: MouseEvent, row: usize, col: usize| {
        if skip_mouseup.get() > 0 {
            set_skip_mouseup.set(skip_mouseup() - 1);
            return;
        }
        if ev.button() == 0 {
            handle_action(PlayAction::Reveal, row, col);
        }
    };
    // TODO - game lifecycle UI (started indicators, ended indicators, countdown / starting alerts, etc.)

    view! {
        <div class="text-center">
            <ActivePlayers/>
            <GameBorder set_active=set_game_is_active>
                {read_signals
                    .into_iter()
                    .enumerate()
                    .map(move |(row, vec)| {
                        view! {
                            <ActiveRow
                                row=row
                                cells=vec
                                set_active_cell=set_active_cell
                                mousedown_handler=handle_mousedown
                                mouseup_handler=handle_mouseup
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
    let game_settings = GameSettings::from(&game_info);
    let board = match game_info.final_board {
        None => vec![vec![PlayerCell::default(); game_info.cols]; game_info.rows],
        Some(b) => b,
    };

    view! {
        <div class="text-center">
            <InactivePlayers players/>
            <GameBorder set_active=move |_| {}>
                {board
                    .into_iter()
                    .enumerate()
                    .map(move |(row, vec)| view! { <InactiveRow row=row cells=vec/> })
                    .collect_view()}
            </GameBorder>
            <ReCreateGame game_settings />
        </div>
    }
}
