pub mod cell;
pub mod client;
mod game;
pub mod players;

use leptos::*;
use leptos_router::*;
use leptos_use::{storage::use_local_storage, utils::JsonCodec};
use serde::{Deserialize, Serialize};

use minesweeper_lib::{cell::PlayerCell, client::ClientPlayer};

use crate::components::{button::Button, input::input_class};
use game::{ActiveGame, InactiveGame};

#[cfg(feature = "ssr")]
use super::FrontendUser;
#[cfg(feature = "ssr")]
use crate::{
    backend::{game_manager::GameManager, users::AuthSession},
    models::game::GameParameters,
};
#[cfg(feature = "ssr")]
use nanoid::nanoid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInfo {
    game_id: String,
    has_owner: bool,
    is_owner: bool,
    rows: usize,
    cols: usize,
    max_players: u8,
    is_started: bool,
    is_completed: bool,
    final_board: Option<Vec<Vec<PlayerCell>>>,
    players: Vec<Option<ClientPlayer>>,
}

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
    let players = players
        .iter()
        .map(|p| ClientPlayer {
            player_id: p.player as usize,
            username: FrontendUser::display_name_or_anon(&p.display_name, p.user.is_some()),
            dead: p.dead,
            score: p.score as usize,
        })
        .fold(vec![None; game.max_players as usize], |mut acc, p| {
            acc[p.player_id] = Some(p.clone());
            acc
        });
    Ok(GameInfo {
        game_id: game.game_id,
        has_owner: game.owner.is_some(),
        is_owner,
        rows: game.rows as usize,
        cols: game.cols as usize,
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

    provide_context::<Resource<String, Result<GameInfo, ServerFnError>>>(game_info);

    let game_view = move |game_info: GameInfo| match game_info.is_completed {
        true => view! { <InactiveGame game_info/> },
        false => view! { <ActiveGame game_info/> },
    };

    view! {
        <Suspense fallback=move || {
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

        </Suspense>
    }
}

fn validate_num_mines(rows: i64, cols: i64, num_mines: i64) -> bool {
    num_mines > 0 && num_mines < rows * cols
}

fn validate_rows(rows: i64) -> bool {
    rows > 0 && rows <= 100
}

fn validate_cols(cols: i64) -> bool {
    cols > 0 && cols <= 100
}

fn validate_num_players(num_players: i64) -> bool {
    num_players > 0 && num_players <= 12
}

#[server(NewGame, "/api")]
async fn new_game(
    rows: i64,
    cols: i64,
    num_mines: i64,
    max_players: i64,
) -> Result<(), ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    if !(validate_num_mines(rows, cols, num_mines)
        && validate_rows(rows)
        && validate_cols(cols)
        && validate_num_players(max_players))
    {
        return Err(ServerFnError::new("Invalid input.".to_string()));
    }

    let id = nanoid!(12);
    game_manager
        .new_game(
            auth_session.user,
            &id,
            GameParameters {
                rows,
                cols,
                num_mines,
                max_players: max_players as u8,
                classic: max_players == 1, // use classic mode (replant) for single player
            },
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    leptos_axum::redirect(&format!("/game/{}/players", id));
    Ok(())
}

#[server(JoinGame, "/api")]
async fn join_game(game_id: String) -> Result<(), ServerFnError> {
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    if !game_manager.game_exists(&game_id).await {
        return Err(ServerFnError::new(format!(
            "Game with game_id {} does not exist",
            game_id
        )));
    }
    leptos_axum::redirect(&format!("/game/{}/players", game_id));
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameSettings {
    rows: i64,
    cols: i64,
    num_mines: i64,
    max_players: i64,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            rows: 50,
            cols: 50,
            num_mines: 500,
            max_players: 8,
        }
    }
}

#[component]
pub fn JoinOrCreateGame() -> impl IntoView {
    let join_game = create_server_action::<JoinGame>();
    let new_game = create_server_action::<NewGame>();

    let (custom_settings, set_custom_settings, _) =
        use_local_storage::<GameSettings, JsonCodec>("custom_game_settings");

    let defaults = GameSettings::default();
    let (rows, set_rows) = create_signal(defaults.rows);
    let (cols, set_cols) = create_signal(defaults.cols);
    let (num_mines, set_num_mines) = create_signal(defaults.num_mines);
    let (max_players, set_max_players) = create_signal(defaults.max_players);
    let (errors, set_errors) = create_signal(Vec::new());

    create_effect(move |_| {
        let stored_settings = custom_settings();
        set_rows(stored_settings.rows);
        set_cols(stored_settings.cols);
        set_num_mines(stored_settings.num_mines);
        set_max_players(stored_settings.max_players);
    });

    create_effect(move |_| {
        let rows = rows();
        let cols = cols();
        let max_mines = num_mines();
        let num_players = max_players();
        let prev_errs = errors();
        let mut errs = Vec::new();
        if !validate_rows(rows) {
            errs.push(String::from("Invalid rows.  Max 100"));
        }
        if !validate_cols(cols) {
            errs.push(String::from("Invalid cols.  Max 100"));
        }
        if !validate_num_players(num_players) {
            errs.push(String::from("Invalid number of players.  Max 12"));
        }
        if !validate_num_mines(rows, cols, max_mines) {
            errs.push(String::from(
                "Invalid number of mines. Must be less than total number of tiles",
            ));
        }
        if errs.len() == prev_errs.len()
            && errs
                .iter()
                .zip(prev_errs.iter())
                .filter(|&(a, b)| a != b)
                .count()
                == 0
        {
            return;
        }
        set_errors(errs);
    });

    // TODO - add presets

    view! {
        <div class="space-y-4 w-80">
            <ActionForm
                action=new_game
                class="w-full max-w-xs space-y-2"
                on:submit=move |ev| {
                    set_custom_settings(GameSettings {
                        rows: rows(),
                        cols: cols(),
                        num_mines: num_mines(),
                        max_players: max_players(),
                    });
                    if !errors().is_empty() {
                        ev.prevent_default();
                    }
                }
            >

                <div class="flex space-x-2">
                    <div class="flex-1">
                        <label
                            class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50"
                            for="new_game_rows"
                        >
                            "Rows:"
                        </label>
                        <input
                            class=input_class("")
                            type="number"
                            id="new_game_rows"
                            name="rows"
                            min=0
                            max=100
                            on:change=move |ev| {
                                set_rows(
                                    event_target_value(&ev).parse::<i64>().unwrap_or_default(),
                                );
                            }

                            prop:value=rows
                        />
                    </div>
                    <div class="flex-1">
                        <label
                            class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50"
                            for="new_game_cols"
                        >
                            "Columns:"
                        </label>
                        <input
                            class=input_class("")
                            type="number"
                            id="new_game_cols"
                            name="cols"
                            min=0
                            max=100
                            on:change=move |ev| {
                                set_cols(
                                    event_target_value(&ev).parse::<i64>().unwrap_or_default(),
                                );
                            }

                            prop:value=cols
                        />
                    </div>
                </div>
                <div class="flex space-x-2">
                    <div class="flex-1">
                        <label
                            class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50"
                            for="new_game_num_mines"
                        >
                            "Mines:"
                        </label>
                        <input
                            class=input_class("")
                            type="number"
                            id="new_game_num_mines"
                            name="num_mines"
                            min=0
                            max=10000
                            on:change=move |ev| {
                                set_num_mines(
                                    event_target_value(&ev).parse::<i64>().unwrap_or_default(),
                                );
                            }

                            prop:value=num_mines
                        />
                    </div>
                    <div class="flex-1">
                        <label
                            class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50"
                            for="new_game_num_players"
                        >
                            "Max Players:"
                        </label>
                        <input
                            class=input_class("")
                            type="number"
                            id="new_game_max_players"
                            name="max_players"
                            min=0
                            max=12
                            on:change=move |ev| {
                                set_max_players(
                                    event_target_value(&ev).parse::<i64>().unwrap_or_default(),
                                );
                            }

                            prop:value=max_players
                        />
                    </div>
                </div>
                <div class="text-red-600 w-full">
                    <For each=errors key=|error| error.to_owned() let:error>
                        <div>{error}</div>
                    </For>
                </div>
                <Button btn_type="submit" class="w-full max-w-xs h-12">
                    "Create New Game"
                </Button>
            </ActionForm>
            <div class="w-full max-w-xs h-6">
                <span class="w-full h-full inline-flex items-center justify-center text-lg font-medium text-gray-800 dark:text-gray-200">
                    <span>"-- or --"</span>
                </span>
            </div>
            <ActionForm action=join_game class="w-full max-w-xs">
                <div class="flex flex-col space-y-2">
                    <label
                        class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50"
                        for="join_game_game_id"
                    >
                        "Join Existing Game:"
                    </label>
                    <div class="flex space-x-2">
                        <input
                            class=input_class("")
                            type="text"
                            placeholder="Enter Game ID"
                            id="join_game_game_id"
                            name="game_id"
                        />
                        <Button btn_type="submit">"Join"</Button>
                    </div>
                </div>
            </ActionForm>
        </div>
    }
}
