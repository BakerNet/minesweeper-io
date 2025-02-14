use leptos::{either::Either, prelude::*};
use serde::{Deserialize, Serialize};

use crate::{button_class, input_class, GameSettings};

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
    pub fn short_name(self) -> &'static str {
        match self {
            Self::ClassicBeginner => "Beginner",
            Self::ClassicIntermediate => "Intermediate",
            Self::ClassicExpert => "Expert",
            Self::SmallMultiplayer => "Small",
            Self::LargeMultiplayer => "Large",
            Self::Custom => "Custom",
        }
    }

    pub fn long_name(self) -> &'static str {
        match self {
            Self::ClassicBeginner => "Classic Beginner",
            Self::ClassicIntermediate => "Classic Intermediate",
            Self::ClassicExpert => "Classic Expert",
            Self::SmallMultiplayer => "Multiplayer Small",
            Self::LargeMultiplayer => "Multiplayer Large",
            Self::Custom => "Custom",
        }
    }
}

impl Default for GameMode {
    fn default() -> Self {
        Self::ClassicBeginner
    }
}

impl From<&GameMode> for GameSettings {
    fn from(value: &GameMode) -> Self {
        match value {
            GameMode::ClassicBeginner => Self {
                rows: 9,
                cols: 9,
                num_mines: 10,
                max_players: 1,
            },
            GameMode::ClassicIntermediate => Self {
                rows: 16,
                cols: 16,
                num_mines: 40,
                max_players: 1,
            },
            GameMode::ClassicExpert => Self {
                rows: 16,
                cols: 30,
                num_mines: 99,
                max_players: 1,
            },
            GameMode::SmallMultiplayer => Self {
                rows: 16,
                cols: 30,
                num_mines: 80,
                max_players: 2,
            },
            GameMode::LargeMultiplayer => Self::default(),
            GameMode::Custom => Self::default(),
        }
    }
}

impl From<GameMode> for GameSettings {
    fn from(value: GameMode) -> Self {
        GameSettings::from(&value)
    }
}

impl From<&GameSettings> for GameMode {
    fn from(value: &GameSettings) -> Self {
        match value {
            GameSettings {
                rows: 9,
                cols: 9,
                num_mines: 10,
                max_players: 1,
            } => Self::ClassicBeginner,
            GameSettings {
                rows: 16,
                cols: 16,
                num_mines: 40,
                max_players: 1,
            } => Self::ClassicIntermediate,
            GameSettings {
                rows: 16,
                cols: 30,
                num_mines: 99,
                max_players: 1,
            } => Self::ClassicExpert,
            GameSettings {
                rows: 16,
                cols: 30,
                num_mines: 80,
                max_players: 2,
            } => Self::SmallMultiplayer,
            GameSettings {
                rows: 50,
                cols: 50,
                num_mines: 500,
                max_players: 8,
            } => Self::LargeMultiplayer,
            _ => Self::Custom,
        }
    }
}

impl From<GameSettings> for GameMode {
    fn from(value: GameSettings) -> Self {
        GameMode::from(&value)
    }
}

#[component]
pub fn PresetButtons(
    selected: Signal<GameMode>,
    set_selected: WriteSignal<GameMode>,
    include_multiplayer: bool,
) -> impl IntoView {
    let multiplayer_modes = [GameMode::SmallMultiplayer, GameMode::LargeMultiplayer];
    let classic_modes = [
        GameMode::ClassicBeginner,
        GameMode::ClassicIntermediate,
        GameMode::ClassicExpert,
    ];

    let class_signal = move |mode: GameMode| {
        let selected = selected.get();
        if mode == selected {
            button_class!(
                "w-full rounded rounded-lg",
                "bg-neutral-800 text-neutral-50 border-neutral-500"
            )
        } else {
            button_class!("w-full rounded rounded-lg")
        }
    };

    let mode_button = move |mode: GameMode| {
        view! {
            <div class="flex-1">
                <button
                    type="button"
                    class=move || class_signal(mode)
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
                "Singleplayer Presets"
            </div>
            <div class="flex w-full space-x-2">{classic_modes.map(mode_button).collect_view()}</div>
        </div>
        {if include_multiplayer {
            Either::Left(
                view! {
                    <div class="w-full space-y-2">
                        <div class="flex-none w-full text-md font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50">
                            "Multiplayer Presets"
                        </div>
                        <div class="flex w-full space-x-2">
                            {multiplayer_modes.map(mode_button).collect_view()}
                        </div>
                    </div>
                },
            )
        } else {
            Either::Right(())
        }}
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
    include_multiplayer: bool,
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
                    class=input_class!()
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
                    class=input_class!()
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
                    class=input_class!()
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
            {if include_multiplayer {
                Either::Left(
                    view! {
                        <div class="flex-1">
                            <label
                                class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 text-neutral-950 dark:text-neutral-50"
                                for="new_game_max_players"
                            >
                                "Max Players:"
                            </label>
                            <input
                                class=input_class!()
                                type="number"
                                id="new_game_max_players"
                                name="max_players"
                                min=0
                                max=12
                                on:change=move |ev| {
                                    set_max_players(
                                        event_target_value(&ev).parse::<i64>().unwrap_or_default(),
                                    );
                                    on_dirty();
                                }
                                prop:value=max_players
                            />
                        </div>
                    },
                )
            } else {
                Either::Right(())
            }}
        </div>
    }
}
