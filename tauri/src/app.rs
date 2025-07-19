use crate::components::{
    GameControls, GameStatsModal, SavedGamesList, TauriActiveGame, TauriInactiveGame,
    TauriReplayGame,
};
use chrono::Utc;
use game_ui::{
    button_class, logo, AnimatedBackground, BackgroundToggle, BackgroundVariant, DarkModeToggle,
    GameInfo, GameMode, GameSettings,
};
use leptos::{either::*, prelude::*, server::codee::string::JsonSerdeWasmCodec};
use leptos_use::storage::use_local_storage;
use minesweeper_lib::game::CompletedMinesweeper;
use std::sync::Arc;

#[derive(Clone)]
pub struct GameData {
    pub game_info: GameInfo,
    pub show_replay: bool,
    pub completed_game: Option<Arc<CompletedMinesweeper>>,
}

impl GameData {
    pub fn new(game_info: GameInfo) -> Self {
        Self {
            game_info,
            show_replay: false,
            completed_game: None,
        }
    }

    pub fn with_completed(
        mut game_info: GameInfo,
        completed_game: Arc<CompletedMinesweeper>,
    ) -> Self {
        // Update game_info to mark as completed
        game_info.is_completed = true;
        game_info.end_time = Some(Utc::now());

        Self {
            game_info,
            show_replay: false,
            completed_game: Some(completed_game),
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let init_game_settings: GameSettings = GameMode::ClassicBeginner.into();
    let (game_signal, set_game_signal) = signal(GameData::new(GameInfo::new_singleplayer(
        String::new(),
        init_game_settings.rows as usize,
        init_game_settings.cols as usize,
        init_game_settings.num_mines as usize,
    )));

    let (show_saved_games, set_show_saved_games) = signal(false);
    let (show_stats, set_show_stats) = signal(false);

    let (background_variant, set_background_variant, _) =
        use_local_storage::<BackgroundVariant, JsonSerdeWasmCodec>("background-variant");

    view! {
        <div class="relative min-h-screen bg-white dark:bg-gray-900">
            <AnimatedBackground variant=background_variant.into() />
            <main class="relative min-h-screen py-8">
                <div>
                    <div class="grid grid-cols-1 md:grid-cols-3 items-center mb-8 gap-4 mx-4">
                        <div class="flex justify-center md:justify-start space-x-2">
                            <button
                                class=button_class!(
                                    "rounded", "bg-purple-700 text-white hover:bg-purple-800/90"
                                )
                                on:click=move |_| set_show_saved_games(true)
                                title="Saved Games"
                            >
                                "Saved Games"
                            </button>
                            <button
                                class=button_class!(
                                    "rounded", "bg-indigo-700 text-white hover:bg-indigo-800/90"
                                )
                                on:click=move |_| set_show_stats(true)
                                title="Stats"
                            >
                                "Stats"
                            </button>
                        </div>
                        <div class="flex justify-center">
                            <h1 class="text-4xl font-bold text-gray-800 dark:text-gray-200">
                                {logo()}
                            </h1>
                        </div>
                        <div class="flex justify-center md:justify-end space-x-2">
                            <BackgroundToggle set_background_variant />
                            <DarkModeToggle />
                        </div>
                    </div>
                    <GameControls set_game_signal />

                    // Game Board
                    <div class="flex-1 flex-col">
                        <div>
                            {move || {
                                let game_data = game_signal.get();
                                if game_data.show_replay && game_data.completed_game.is_some() {
                                    EitherOf3::A(
                                        view! { <TauriReplayGame game_data set_game_signal /> },
                                    )
                                } else if game_data.game_info.is_completed {
                                    EitherOf3::B(
                                        view! { <TauriInactiveGame game_data set_game_signal /> },
                                    )
                                } else {
                                    EitherOf3::C(
                                        view! {
                                            <TauriActiveGame
                                                game_info=game_data.game_info
                                                set_game_signal
                                            />
                                        },
                                    )
                                }
                            }}
                        </div>
                    </div>

                    // Saved Games Modal
                    <Show when=show_saved_games>
                        <div
                            class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4"
                            on:click=move |_| set_show_saved_games(false)
                        >
                            <div
                                class="bg-white dark:bg-gray-800 rounded-lg shadow-xl border-2 border-gray-300 dark:border-gray-600 w-auto max-w-4xl max-h-[80vh] overflow-auto"
                                on:click=move |e| e.stop_propagation()
                            >
                                <SavedGamesList set_game_signal set_show_saved_games />
                            </div>
                        </div>
                    </Show>

                    // Stats Modal
                    <Show when=show_stats>
                        <div
                            class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center p-4"
                            on:click=move |_| set_show_stats(false)
                        >
                            <div
                                class="bg-white dark:bg-gray-800 rounded-lg shadow-xl border-2 border-gray-300 dark:border-gray-600 w-auto max-w-4xl max-h-[80vh] overflow-auto"
                                on:click=move |e| e.stop_propagation()
                            >
                                <GameStatsModal set_show_stats />
                            </div>
                        </div>
                    </Show>
                </div>
            </main>
        </div>
    }
}
