mod cell;
pub mod client;
mod game;
pub mod players;

use leptos::*;
use leptos_router::*;
use serde::{Deserialize, Serialize};

use minesweeper_lib::cell::PlayerCell;

use client::FrontendGame;
use game::{ActiveGame, InactiveGame};

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
