use codee::string::JsonSerdeWasmCodec;
use leptos::prelude::*;
use leptos_use::storage::{use_local_storage, use_local_storage_with_options, UseStorageOptions};
use wasm_bindgen::JsValue;

use game_ui::{button_class, input_class, GameMode, GameSettings, PresetButtons, SettingsInputs};

#[cfg(feature = "ssr")]
use game_manager::{models::GameParameters, GameManager};
#[cfg(feature = "ssr")]
use nanoid::nanoid;
#[cfg(feature = "ssr")]
use web_auth::AuthSession;

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

#[server]
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
            },
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    leptos_axum::redirect(&format!("/game/{id}"));
    Ok(())
}

#[server]
async fn join_game(game_id: String) -> Result<(), ServerFnError> {
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    if !game_manager.game_exists(&game_id).await {
        return Err(ServerFnError::new(format!(
            "Game with game_id {game_id} does not exist"
        )));
    }
    leptos_axum::redirect(&format!("/game/{game_id}"));
    Ok(())
}

#[component]
pub fn JoinOrCreateGame() -> impl IntoView {
    let join_game = ServerAction::<JoinGame>::new();
    let new_game = ServerAction::<NewGame>::new();

    let storage_options = UseStorageOptions::<GameMode, serde_json::Error, JsValue>::default()
        .delay_during_hydration(true);
    let (selected_mode, set_selected_mode, _) = use_local_storage_with_options::<
        GameMode,
        JsonSerdeWasmCodec,
    >("game_mode_settings", storage_options);

    let (custom_settings, set_custom_settings, _) =
        use_local_storage::<GameSettings, JsonSerdeWasmCodec>("custom_game_settings");

    let defaults = GameSettings::default();
    let (rows, set_rows) = signal(defaults.rows);
    let (cols, set_cols) = signal(defaults.cols);
    let (num_mines, set_num_mines) = signal(defaults.num_mines);
    let (max_players, set_max_players) = signal(defaults.max_players);
    let (dirty, set_dirty) = signal(false);
    let (errors, set_errors) = signal(Vec::new());

    let load_custom_settings = move || {
        let stored_settings = custom_settings.get_untracked();
        set_rows(stored_settings.rows);
        set_cols(stored_settings.cols);
        set_num_mines(stored_settings.num_mines);
        set_max_players(stored_settings.max_players);
    };

    Effect::watch(
        selected_mode,
        move |mode, _, _| {
            if *mode != GameMode::Custom {
                let mode_settings = GameSettings::from(mode);
                set_rows(mode_settings.rows);
                set_cols(mode_settings.cols);
                set_num_mines(mode_settings.num_mines);
                set_max_players(mode_settings.max_players);
                set_dirty(false);
            } else if !dirty.get_untracked() {
                load_custom_settings();
            }
        },
        true,
    );

    Effect::new(move |_| {
        let rows = rows.get();
        let cols = cols.get();
        let max_mines = num_mines.get();
        let num_players = max_players.get();
        let prev_errs = errors.get();
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

    view! {
        <div class="space-y-4 w-80">
            <ActionForm
                action=new_game
                attr:class="w-full max-w-xs space-y-2"
                on:submit=move |ev| {
                    if selected_mode() == GameMode::Custom {
                        set_custom_settings(GameSettings {
                            rows: rows(),
                            cols: cols(),
                            num_mines: num_mines(),
                            max_players: max_players(),
                        });
                    }
                    if !errors().is_empty() {
                        ev.prevent_default();
                    }
                }
            >

                <div class="space-y-2">
                    <PresetButtons
                        selected=selected_mode
                        set_selected=set_selected_mode
                        include_multiplayer=true
                    />
                </div>
                <div class="space-y-2">
                    <SettingsInputs
                        rows
                        set_rows
                        cols
                        set_cols
                        num_mines
                        set_num_mines
                        max_players
                        set_max_players
                        on_dirty=move || {
                            set_dirty(true);
                            set_selected_mode(GameMode::Custom);
                        }
                        include_multiplayer=true
                    />

                </div>
                <div class="text-red-600 w-full">
                    <For each=errors key=|error| error.to_owned() let:error>
                        <div>{error}</div>
                    </For>
                </div>
                <button
                    type="submit"
                    class=button_class!(
                        "w-full max-w-xs h-12", "text-neutral-50 bg-sky-700 hover:bg-sky-900/90"
                    )
                    disabled=new_game.pending()
                >
                    "Create New Game"
                </button>
            </ActionForm>
            <div class="w-full max-w-xs h-6">
                <span class="w-full h-full inline-flex items-center justify-center text-lg font-medium text-gray-800 dark:text-gray-200">
                    <span>"-- or --"</span>
                </span>
            </div>
            <ActionForm action=join_game attr:class="w-full max-w-xs">
                <div class="flex flex-col space-y-2">
                    <label
                        class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50"
                        for="join_game_game_id"
                    >
                        "Join Existing Game:"
                    </label>
                    <div class="flex space-x-2">
                        <input
                            class=input_class!()
                            type="text"
                            placeholder="Enter Game ID"
                            id="join_game_game_id"
                            name="game_id"
                        />
                        <button type="submit" class=button_class!() disabled=join_game.pending()>
                            "Join"
                        </button>
                    </div>
                </div>
                <Show when=move || join_game.value().with(|val| matches!(val, Some(Err(_))))>
                    <div class="text-red-600 w-full">"Game does not exist"</div>
                </Show>
            </ActionForm>
        </div>
    }
}

#[component]
pub fn ReCreateGameButton(game_settings: GameSettings) -> impl IntoView {
    let new_game = ServerAction::<NewGame>::new();

    view! {
        <ActionForm action=new_game>
            <input type="hidden" name="rows" prop:value=game_settings.rows />
            <input type="hidden" name="cols" prop:value=game_settings.cols />
            <input type="hidden" name="num_mines" prop:value=game_settings.num_mines />
            <input type="hidden" name="max_players" prop:value=game_settings.max_players />
            <button
                type="submit"
                class=button_class!(
                    "rounded",
                    "bg-green-700 hover:bg-green-800/90 text-white"
                )
                disabled=new_game.pending()
            >
                "Play Again"
            </button>
        </ActionForm>
    }
}
