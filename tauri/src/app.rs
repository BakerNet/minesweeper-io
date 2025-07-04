use crate::game::FrontendGame;
use game_ui::{
    ActiveGame, ActiveMines, ActiveTimer, GameInfo, GameMode, GameSettings, GameStateWidget,
    GameWidgets,
};
use leptos::prelude::*;
use minesweeper_lib::game::Action as PlayAction;

#[component]
pub fn App() -> impl IntoView {
    let init_game_settings: GameSettings = GameMode::ClassicBeginner.into();
    let (game_signal, set_game_signal) = signal(GameInfo::new_singleplayer(
        String::new(),
        init_game_settings.rows as usize,
        init_game_settings.cols as usize,
        init_game_settings.num_mines as usize,
    ));

    let new_game = move |game_mode: GameMode| {
        let game_settings: GameSettings = game_mode.into();
        let game_info = GameInfo::new_singleplayer(
            String::new(),
            game_settings.rows as usize,
            game_settings.cols as usize,
            game_settings.num_mines as usize,
        );
        set_game_signal(game_info);
    };

    view! {
        <main class="min-h-screen bg-gray-100 py-8">
            <div>
                <div class="text-center mb-8">
                    <h1 class="text-4xl font-bold text-gray-800 mb-4">"Minesweeper"</h1>

                    // Game Controls
                    <div class="flex justify-center space-x-4 mb-6">
                        <button
                            class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600"
                            on:click=move |_| new_game(GameMode::ClassicBeginner)
                            title="Start Beginner Game"
                        >
                            "Beginner (9×9)"
                        </button>
                        <button
                            class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
                            on:click=move |_| new_game(GameMode::ClassicIntermediate)
                            title="Start Intermediate Game"
                        >
                            "Intermediate (16×16)"
                        </button>
                        <button
                            class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600"
                            on:click=move |_| new_game(GameMode::ClassicExpert)
                            title="Start Expert Game"
                        >
                            "Expert (16×30)"
                        </button>
                        <button
                            class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600"
                            on:click=move |_| {
                                let curr_game = game_signal.get_untracked();
                                let curr_settings = GameSettings::from(curr_game);
                                new_game(curr_settings.into())
                            }
                            title="Restart Game"
                        >
                            "Restart"
                        </button>
                    </div>
                </div>

                // Game Board
                <div class="flex-1 flex-col">
                    {
                        move || {
                            let game_info = game_signal.get();
                            view!{<TauriActiveGame game_info />}
                        }
                    }
                </div>
            </div>
        </main>
    }
}

#[component]
fn TauriActiveGame(game_info: GameInfo) -> impl IntoView {
    let (error, set_error) = signal::<Option<String>>(None);

    let game = FrontendGame::new(&game_info, set_error);
    let flag_count = game.flag_count;
    let sync_time = game.sync_time;
    let completed = game.completed;
    let victory = game.victory;
    let dead = game.dead;
    let cells = (*game.cells).clone();

    let game = StoredValue::new(game);

    let handle_action = move |pa: PlayAction, row: usize, col: usize| {
        game.with_value(|game| {
            let res = match pa {
                PlayAction::Reveal => game.try_reveal(row, col),
                PlayAction::Flag => game.try_flag(row, col),
                PlayAction::RevealAdjacent => game.try_reveal_adjacent(row, col),
            };
            res.unwrap_or_else(|e| (game.err_signal)(Some(format!("{e:?}"))));
        })
    };

    view! {
        <GameWidgets>
            <ActiveMines num_mines=game_info.num_mines flag_count />
            <GameStateWidget completed victory dead sync_time />
            <ActiveTimer sync_time completed />
        </GameWidgets>
        <ActiveGame cell_read_signals=cells action_handler=handle_action />
        <div class="text-red-600 h-8 text-center">{error}</div>
    }
}
