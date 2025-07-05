use crate::components::{GameControls, TauriActiveGame, TauriInactiveGame, TauriReplayGame};
use chrono::Utc;
use game_ui::{
    logo, AnimatedBackground, BackgroundToggle, BackgroundVariant, DarkModeToggle, GameInfo,
    GameMode, GameSettings,
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

    pub fn with_completed(mut game_info: GameInfo, completed_game: CompletedMinesweeper) -> Self {
        // Update game_info to mark as completed
        game_info.is_completed = true;
        game_info.end_time = Some(Utc::now());

        Self {
            game_info,
            show_replay: false,
            completed_game: Some(Arc::new(completed_game)),
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

    let (background_variant, set_background_variant, _) =
        use_local_storage::<BackgroundVariant, JsonSerdeWasmCodec>("background-variant");

    view! {
        <div class="relative min-h-screen bg-white dark:bg-gray-900">
        <AnimatedBackground variant=background_variant.into() />
        <main class="relative min-h-screen py-8">
            <div>
                <div class="grid grid-cols-1 md:grid-cols-3 items-center mb-8 gap-4 mx-4">
                    <div></div>
                    <div class="flex justify-center">
                        <h1 class="text-4xl font-bold text-gray-800 dark:text-gray-200">{logo()}</h1>
                    </div>
                    <div class="flex justify-center md:justify-end space-x-2">
                        <BackgroundToggle set_background_variant />
                        <DarkModeToggle />
                    </div>
                </div>
                <GameControls set_game_signal />

                // Game Board
                <div class="flex-1 flex-col">
                    {move || {
                        let game_data = game_signal.get();
                        if game_data.show_replay && game_data.completed_game.is_some() {
                            EitherOf3::A(view! { <TauriReplayGame game_data set_game_signal /> })
                        } else if game_data.game_info.is_completed {
                            EitherOf3::B(view! { <TauriInactiveGame game_info=game_data.game_info set_game_signal /> })
                        } else {
                            EitherOf3::C(view! { <TauriActiveGame game_info=game_data.game_info set_game_signal /> })
                        }
                    }}
                </div>
            </div>
        </main>
        </div>
    }
}
