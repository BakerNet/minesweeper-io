mod cell;
pub mod client;
pub mod players;

use anyhow::Result;
use leptos::*;
use leptos_router::*;
use leptos_use::{core::ConnectionReadyState, use_websocket, UseWebsocketReturn};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use minesweeper::cell::PlayerCell;

use cell::{ActiveRow, InactiveRow};
use client::FrontendGame;

#[cfg(feature = "ssr")]
use crate::backend::{game_manager::GameManager, users::AuthSession};
#[cfg(feature = "ssr")]
use nanoid::nanoid;

use super::FrontendUser;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    game_id: String,
    is_owner: bool,
    rows: usize,
    cols: usize,
    max_players: u8,
    is_started: bool,
    is_completed: bool,
    final_board: Option<Vec<Vec<PlayerCell>>>,
}

#[server(GetGame, "/api")]
pub async fn get_game(game_id: String) -> Result<GameInfo, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::ServerError("Unable to find auth session".to_string()))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::ServerError("No game manager".to_string()))?;
    let game = game_manager
        .get_game(&game_id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let is_owner = if let Some(user) = auth_session.user {
        user.id == game.owner
    } else {
        false
    };
    Ok(GameInfo {
        game_id: game.game_id,
        is_owner,
        rows: game.rows as usize,
        cols: game.cols as usize,
        max_players: game.max_players,
        is_started: game.is_started,
        is_completed: game.is_completed,
        final_board: game.final_board,
    })
}

#[component]
pub fn Game() -> impl IntoView {
    // TODO - game_id should be parameter, and there should be a parent component that renders
    // Games based on id param (with Suspense)
    let params = use_params_map();
    let game_id = move || params.get().get("id").cloned().unwrap_or_default();
    let game_info = create_resource(game_id, get_game);

    provide_context::<Resource<String, Result<GameInfo, ServerFnError>>>(game_info);

    let game_view = move |game_info: GameInfo| match game_info.is_completed {
        true => view! { <InactiveGame game_info/> },
        false => view! { <ActiveGame game_info/> },
    };

    view! {
        <Suspense fallback=move || {
            view! { <div>"Loading..."</div> }
        }>
            {game_info
                .get()
                .map(|game_info| {
                    view! {
                        <ErrorBoundary fallback=|_| {
                            view! { <div class="error">"Game not found"</div> }
                        }>{move || { game_info.clone().map(game_view) }}</ErrorBoundary>
                    }
                })}

        </Suspense>
    }
}

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
    } = use_websocket(&format!(
        "ws://localhost:3000/api/websocket/game/{}",
        &game_info.game_id
    ));

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
        <div>do you see me?</div>
            <div class="error">{error}</div>
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

#[server(StartGame, "/api")]
async fn start_game() -> Result<(), ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::ServerError("Unable to find auth session".to_string()))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::ServerError("No game manager".to_string()))?;

    let user = match auth_session.user {
        Some(user) => user,
        None => {
            return Err(ServerFnError::ServerError("Not logged in".to_string()));
        }
    };

    let id = nanoid!(12);
    game_manager
        .new_game(&user, &id, 50, 50, 500, 8)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    leptos_axum::redirect(&format!("/game/{}", id));
    Ok(())
}

#[server(JoinGame, "/api")]
async fn join_game(game_id: String) -> Result<(), ServerFnError> {
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::ServerError("No game manager".to_string()))?;
    if !game_manager.game_exists(&game_id).await {
        return Err(ServerFnError::ServerError(format!(
            "Game with game_id {} does not exist",
            game_id
        )));
    }
    leptos_axum::redirect(&format!("/game/{}", game_id));
    Ok(())
}

#[component]
pub fn StartGame<S>(user: Resource<S, Option<FrontendUser>>) -> impl IntoView
where
    S: PartialEq + Clone + 'static,
{
    let join_game = create_server_action::<JoinGame>();
    let new_game = create_server_action::<StartGame>();

    view! {
        <div id="StartGame">
            <ActionForm action=join_game>
                <label for="game_id">Game ID:</label>
                <input type="text" name="game_id"/>
                <button type="submit">"Join Game"</button>
            </ActionForm>
            <Transition fallback=move || {
                view! {}
            }>
                {user()
                    .flatten()
                    .map(|_| {
                        view! {
                            <ActionForm action=new_game>
                                <button type="submit">"New Game"</button>
                            </ActionForm>
                        }
                    })}

            </Transition>
        </div>
    }
}
