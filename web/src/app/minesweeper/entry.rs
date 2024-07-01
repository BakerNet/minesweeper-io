use leptos::*;
use leptos_router::*;
use leptos_use::{storage::use_local_storage, utils::JsonCodec};
use serde::{Deserialize, Serialize};

use crate::components::{button_class, input_class};

use super::GameSettings;

#[cfg(feature = "ssr")]
use crate::{
    backend::{AuthSession, GameManager},
    models::game::GameParameters,
};
#[cfg(feature = "ssr")]
use nanoid::nanoid;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum GameMode {
    ClassicBeginner,
    ClassicIntermediate,
    ClassicExpert,
    SmallMultiplayer,
    LargeMultiplayer,
    Custom,
}

impl GameMode {
    fn short_name(self) -> &'static str {
        match self {
            GameMode::ClassicBeginner => "Beginner",
            GameMode::ClassicIntermediate => "Intermediate",
            GameMode::ClassicExpert => "Expert",
            GameMode::SmallMultiplayer => "Small",
            GameMode::LargeMultiplayer => "Large",
            GameMode::Custom => "Custom",
        }
    }
}

impl Default for GameMode {
    fn default() -> Self {
        Self::LargeMultiplayer
    }
}

impl From<&GameMode> for GameSettings {
    fn from(val: &GameMode) -> Self {
        match val {
            GameMode::ClassicBeginner => GameSettings {
                rows: 9,
                cols: 9,
                num_mines: 10,
                max_players: 1,
            },
            GameMode::ClassicIntermediate => GameSettings {
                rows: 16,
                cols: 16,
                num_mines: 40,
                max_players: 1,
            },
            GameMode::ClassicExpert => GameSettings {
                rows: 16,
                cols: 30,
                num_mines: 99,
                max_players: 1,
            },
            GameMode::SmallMultiplayer => GameSettings {
                rows: 16,
                cols: 30,
                num_mines: 80,
                max_players: 2,
            },
            GameMode::LargeMultiplayer => GameSettings::default(),
            GameMode::Custom => GameSettings::default(),
        }
    }
}

impl From<GameMode> for GameSettings {
    fn from(value: GameMode) -> Self {
        GameSettings::from(&value)
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
            },
        )
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    leptos_axum::redirect(&format!("/game/{}", id));
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
    leptos_axum::redirect(&format!("/game/{}", game_id));
    Ok(())
}

#[component]
pub fn PresetButtons(
    selected: Signal<GameMode>,
    set_selected: WriteSignal<GameMode>,
) -> impl IntoView {
    let multiplayer_modes = [GameMode::SmallMultiplayer, GameMode::LargeMultiplayer];
    let classic_modes = [
        GameMode::ClassicBeginner,
        GameMode::ClassicIntermediate,
        GameMode::ClassicExpert,
    ];

    let mode_button = move |mode: GameMode| {
        view! {
            <div class="flex-1">
                <button
                    type="button"
                    class=move || {
                        let selected_colors = if selected() == mode {
                            Some("bg-neutral-800 text-neutral-50 border-neutral-500")
                        } else {
                            None
                        };
                        button_class(Some("w-full rounded rounded-lg"), selected_colors)
                    }

                    on:click=move |_| {
                        set_selected(mode);
                    }
                >

                    {mode.short_name()}
                </button>
            </div>
        }
    };

    view! {
        <div class="w-full space-y-2">
            <div class="flex-none w-full text-md font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50">
                "Multiplayer Presets"
            </div>
            <div class="flex w-full space-x-2">
                {multiplayer_modes.map(mode_button).collect_view()}
            </div>
        </div>
        <div class="w-full space-y-2">
            <div class="flex-none w-full text-md font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50">
                "Classic Presets"
            </div>
            <div class="flex w-full space-x-2">{classic_modes.map(mode_button).collect_view()}</div>
        </div>
        <div class="w-full space-y-2">
            <div class="flex-none w-full text-md font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50">
                "Custom"
            </div>
            <div class="flex w-full space-x-2">{mode_button(GameMode::Custom)}</div>
        </div>
    }
}

#[component]
pub fn SettingsInputs<F>(
    rows: ReadSignal<i64>,
    set_rows: WriteSignal<i64>,
    cols: ReadSignal<i64>,
    set_cols: WriteSignal<i64>,
    num_mines: ReadSignal<i64>,
    set_num_mines: WriteSignal<i64>,
    max_players: ReadSignal<i64>,
    set_max_players: WriteSignal<i64>,
    on_dirty: F,
) -> impl IntoView
where
    F: Fn() + Clone + Copy + 'static,
{
    view! {
        <div class="flex space-x-2">
            <div class="flex-1">
                <label
                    class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50"
                    for="new_game_rows"
                >
                    "Rows:"
                </label>
                <input
                    class=input_class(None)
                    type="number"
                    id="new_game_rows"
                    name="rows"
                    min=0
                    max=100
                    on:change=move |ev| {
                        set_rows(event_target_value(&ev).parse::<i64>().unwrap_or_default());
                        on_dirty();
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
                    class=input_class(None)
                    type="number"
                    id="new_game_cols"
                    name="cols"
                    min=0
                    max=100
                    on:change=move |ev| {
                        set_cols(event_target_value(&ev).parse::<i64>().unwrap_or_default());
                        on_dirty();
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
                    class=input_class(None)
                    type="number"
                    id="new_game_num_mines"
                    name="num_mines"
                    min=0
                    max=10000
                    on:change=move |ev| {
                        set_num_mines(event_target_value(&ev).parse::<i64>().unwrap_or_default());
                        on_dirty();
                    }

                    prop:value=num_mines
                />
            </div>
            <div class="flex-1">
                <label
                    class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50"
                    for="new_game_max_players"
                >
                    "Max Players:"
                </label>
                <input
                    class=input_class(None)
                    type="number"
                    id="new_game_max_players"
                    name="max_players"
                    min=0
                    max=12
                    on:change=move |ev| {
                        set_max_players(event_target_value(&ev).parse::<i64>().unwrap_or_default());
                        on_dirty();
                    }

                    prop:value=max_players
                />
            </div>
        </div>
    }
}

#[component]
pub fn JoinOrCreateGame() -> impl IntoView {
    let join_game = create_server_action::<JoinGame>();
    let new_game = create_server_action::<NewGame>();

    let (selected_mode, set_selected_mode, _) =
        use_local_storage::<GameMode, JsonCodec>("game_mode_settings");

    let (custom_settings, set_custom_settings, _) =
        use_local_storage::<GameSettings, JsonCodec>("custom_game_settings");

    let defaults = GameSettings::default();
    let (rows, set_rows) = create_signal(defaults.rows);
    let (cols, set_cols) = create_signal(defaults.cols);
    let (num_mines, set_num_mines) = create_signal(defaults.num_mines);
    let (max_players, set_max_players) = create_signal(defaults.max_players);
    let (dirty, set_dirty) = create_signal(false);
    let (errors, set_errors) = create_signal(Vec::new());

    let load_custom_settings = move || {
        let stored_settings = custom_settings();
        set_rows(stored_settings.rows);
        set_cols(stored_settings.cols);
        set_num_mines(stored_settings.num_mines);
        set_max_players(stored_settings.max_players);
    };

    create_effect(move |_| {
        let stored_mode = selected_mode();
        if stored_mode != GameMode::Custom {
            let mode_settings = GameSettings::from(stored_mode);
            set_rows(mode_settings.rows);
            set_cols(mode_settings.cols);
            set_num_mines(mode_settings.num_mines);
            set_max_players(mode_settings.max_players);
            set_dirty(false);
        } else if !dirty() {
            load_custom_settings();
        }
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

    view! {
        <div class="space-y-4 w-80">
            <ActionForm
                action=new_game
                class="w-full max-w-xs space-y-2"
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
                    <PresetButtons selected=selected_mode set_selected=set_selected_mode/>
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
                        on_dirty = move || {
                            set_dirty(true);
                            set_selected_mode(GameMode::Custom);
                        }
                    />
                </div>
                <div class="text-red-600 w-full">
                    <For each=errors key=|error| error.to_owned() let:error>
                        <div>{error}</div>
                    </For>
                </div>
                <button
                    type="submit"
                    class=button_class(Some("w-full max-w-xs h-12"), None)
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
                            class=input_class(None)
                            type="text"
                            placeholder="Enter Game ID"
                            id="join_game_game_id"
                            name="game_id"
                        />
                        <button
                            type="submit"
                            class=button_class(None, None)
                            disabled=join_game.pending()
                        >
                            "Join"
                        </button>
                    </div>
                </div>
            </ActionForm>
        </div>
    }
}

#[component]
pub fn ReCreateGame(game_settings: GameSettings) -> impl IntoView {
    let new_game = create_server_action::<NewGame>();

    view! {
        <div class="flex flex-col items-center space-y-4">
            <ActionForm
                action=new_game
                class="w-full max-w-xs space-y-2"
            >
                <input
                    type="hidden"
                    name="rows"
                    prop:value=game_settings.rows
                />
                <input
                    type="hidden"
                    name="cols"
                    prop:value=game_settings.cols
                />
                <input
                    type="hidden"
                    name="num_mines"
                    prop:value=game_settings.num_mines
                />
                <input
                    type="hidden"
                    name="max_players"
                    prop:value=game_settings.max_players
                />
                <button
                    type="submit"
                    class=button_class(Some("w-full max-w-xs h-8"), Some("bg-green-700 hover:bg-green-800/90 text-white"))
                    disabled=new_game.pending()
                >
                    "Play Again"
                </button>
            </ActionForm>
        </div>
    }
}
