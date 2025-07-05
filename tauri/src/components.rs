use crate::game::FrontendGame;
use crate::GameData;
use chrono::Utc;
use game_ui::{
    game_time_from_start_end, ActiveGame, ActiveMines, ActiveTimer, GameInfo, GameMode,
    GameSettings, GameState, GameStateWidget, GameWidgets, InactiveGame, InactiveGameStateWidget,
    InactiveMines, InactiveTimer, ReplayControls, ReplayGame,
};
use leptos::prelude::*;
use minesweeper_lib::{
    analysis::AnalyzedCell,
    board::CompactBoard,
    cell::{HiddenCell, PlayerCell},
    client::ClientPlayer,
    game::Action as PlayAction,
    replay::ReplayAnalysisCell,
};
use std::sync::Arc;

#[component]
pub fn GameControls(set_game_signal: WriteSignal<GameData>) -> impl IntoView {
    let (show_custom, set_show_custom) = signal(false);
    let (custom_rows, set_custom_rows) = signal(9i64);
    let (custom_cols, set_custom_cols) = signal(9i64);
    let (custom_mines, set_custom_mines) = signal(10i64);
    let (custom_errors, set_custom_errors) = signal::<Vec<String>>(Vec::new());

    let new_game = move |game_settings: GameSettings| {
        let game_info = GameInfo::new_singleplayer(
            String::new(),
            game_settings.rows as usize,
            game_settings.cols as usize,
            game_settings.num_mines as usize,
        );
        set_game_signal(GameData::new(game_info));
    };

    let new_custom_game = move || {
        let rows = custom_rows.get();
        let cols = custom_cols.get();
        let mines = custom_mines.get();

        let mut errors = Vec::new();
        if rows <= 0 || rows > 100 {
            errors.push("Invalid rows. Must be between 1 and 100".to_string());
        }
        if cols <= 0 || cols > 100 {
            errors.push("Invalid columns. Must be between 1 and 100".to_string());
        }
        if mines <= 0 || mines >= rows * cols {
            errors.push("Invalid mines. Must be less than total tiles".to_string());
        }

        if errors.is_empty() {
            let game_info = GameInfo::new_singleplayer(
                String::new(),
                rows as usize,
                cols as usize,
                mines as usize,
            );
            set_game_signal(GameData::new(game_info));
            set_show_custom(false);
        } else {
            set_custom_errors(errors);
        }
    };

    view! {
        <div class="text-center mb-8">
            // Game Controls
            <div class="flex justify-center space-x-4 mb-6">
                <button
                    class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 dark:bg-blue-600 dark:hover:bg-blue-700 cursor-pointer"
                    on:click=move |_| {
                        new_game(GameMode::ClassicBeginner.into());
                        set_show_custom(false);
                    }
                    title="Start Beginner Game"
                >
                    "Beginner"
                </button>
                <button
                    class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 dark:bg-green-600 dark:hover:bg-green-700 cursor-pointer"
                    on:click=move |_| {
                        new_game(GameMode::ClassicIntermediate.into());
                        set_show_custom(false);
                    }
                    title="Start Intermediate Game"
                >
                    "Intermediate"
                </button>
                <button
                    class="px-4 py-2 bg-red-500 text-white rounded hover:bg-red-600 dark:bg-red-600 dark:hover:bg-red-700 cursor-pointer"
                    on:click=move |_| {
                        new_game(GameMode::ClassicExpert.into());
                        set_show_custom(false);
                    }
                    title="Start Expert Game"
                >
                    "Expert"
                </button>
                <button
                    class="px-4 py-2 bg-purple-500 text-white rounded hover:bg-purple-600 dark:bg-purple-600 dark:hover:bg-purple-700 cursor-pointer"
                    on:click=move |_| {
                        set_show_custom(!show_custom.get());
                        set_custom_errors(Vec::new());
                    }
                    title="Custom Game"
                >
                    "Custom"
                </button>
            </div>

            // Custom game inputs
            <Show when=show_custom>
                <div class="max-w-sm mx-auto space-y-4 p-4 bg-gray-100 dark:bg-gray-800 rounded-lg">
                    <div class="flex space-x-4">
                        <div class="flex-1">
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Rows"
                            </label>
                            <input
                                type="number"
                                min="1"
                                max="100"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                                prop:value=custom_rows
                                on:input=move |ev| {
                                    set_custom_rows(event_target_value(&ev).parse::<i64>().unwrap_or(9));
                                    set_custom_errors(Vec::new());
                                }
                            />
                        </div>
                        <div class="flex-1">
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Columns"
                            </label>
                            <input
                                type="number"
                                min="1"
                                max="100"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                                prop:value=custom_cols
                                on:input=move |ev| {
                                    set_custom_cols(event_target_value(&ev).parse::<i64>().unwrap_or(9));
                                    set_custom_errors(Vec::new());
                                }
                            />
                        </div>
                        <div class="flex-1">
                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                                "Mines"
                            </label>
                            <input
                                type="number"
                                min="1"
                                max="9999"
                                class="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
                                prop:value=custom_mines
                                on:input=move |ev| {
                                    set_custom_mines(event_target_value(&ev).parse::<i64>().unwrap_or(10));
                                    set_custom_errors(Vec::new());
                                }
                            />
                        </div>
                    </div>
                    <Show
                        when=move || !custom_errors.get().is_empty()
                        fallback=|| view! { <div></div> }
                    >
                        <div class="text-red-600 text-sm space-y-1">
                            {move || custom_errors.get().into_iter().map(|err| view! { <div>{err}</div> }).collect_view()}
                        </div>
                    </Show>
                    <button
                        class="w-full px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600 dark:bg-green-600 dark:hover:bg-green-700 cursor-pointer"
                        on:click=move |_| new_custom_game()
                    >
                        "Start Custom Game"
                    </button>
                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn TauriActiveGame(
    game_info: GameInfo,
    set_game_signal: WriteSignal<GameData>,
) -> impl IntoView {
    let (error, set_error) = signal::<Option<String>>(None);

    let game = FrontendGame::new(&game_info, set_error);
    let flag_count = game.flag_count;
    let sync_time = game.sync_time;
    let completed = game.completed;
    let victory = game.victory;
    let dead = game.dead;
    let cells = (*game.cells).clone();
    let num_mines = game_info.num_mines;

    let game_info = StoredValue::new(game_info);
    let game = StoredValue::new(game);

    // Watch for game completion
    Effect::new(move |prev_completed: Option<bool>| {
        let is_completed = completed.get();
        log::debug!("Hello from completed effect");
        if is_completed && prev_completed != Some(true) {
            log::debug!("Hello from completed 0");
            // Game just completed, extract the CompletedMinesweeper
            game.with_value(|g| {
                log::debug!("Hello from completed 1");
                if let Some(completed_minesweeper) = g.extract_completed_game() {
                    log::debug!("Hello from completed 2");
                    let mut new_game_info = game_info.get_value().clone();
                    new_game_info.is_completed = true;
                    new_game_info.is_started = true;
                    new_game_info.start_time = g.start_time.get();
                    new_game_info.end_time = Some(Utc::now());
                    new_game_info.final_board =
                        CompactBoard::from_board(&completed_minesweeper.player_board_final(0));
                    new_game_info.players = vec![Some(ClientPlayer {
                        player_id: 0,
                        username: String::new(),
                        dead: dead.get_untracked(),
                        victory_click: victory.get_untracked(),
                        top_score: false,
                        score: 0,
                    })];
                    let updated_data =
                        GameData::with_completed(new_game_info, completed_minesweeper);
                    set_game_signal(updated_data);
                }
            });
        }
        is_completed
    });

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

    let remake_game = move || {
        let game_info = game_info.get_value();
        set_game_signal.set(GameData::new(GameInfo::new_singleplayer(
            String::new(),
            game_info.rows,
            game_info.cols,
            game_info.num_mines,
        )));
    };

    view! {
        <GameWidgets>
            <ActiveMines num_mines flag_count />
            <GameStateWidget victory dead sync_time on_click=remake_game />
            <ActiveTimer sync_time completed />
        </GameWidgets>
        <ActiveGame cell_read_signals=cells action_handler=handle_action />
        <div class="text-red-600 h-8 text-center">{error}</div>
    }
}

#[component]
pub fn TauriInactiveGame(
    game_info: GameInfo,
    set_game_signal: WriteSignal<GameData>,
) -> impl IntoView {
    let game_time = game_time_from_start_end(game_info.start_time, game_info.end_time);
    let game_state = game_info
        .players
        .first()
        .flatten()
        .map(|p| {
            if p.victory_click {
                GameState::Victory
            } else if p.dead {
                GameState::Dead
            } else {
                GameState::NotStarted
            }
        })
        .unwrap_or(GameState::NotStarted);

    // Get the final board from completed game or fall back to game_info board
    let board = game_info.final_board.to_board();

    let game_info = StoredValue::new(game_info);

    // Count mines in the final board
    let num_mines = board
        .rows_iter()
        .flatten()
        .filter(|&c| matches!(c, PlayerCell::Hidden(HiddenCell::Mine)))
        .count();

    let remake_game = move || {
        let game_info = game_info.get_value();
        set_game_signal.set(GameData::new(GameInfo::new_singleplayer(
            String::new(),
            game_info.rows,
            game_info.cols,
            game_info.num_mines,
        )));
    };

    let open_replay = move |_| {
        set_game_signal.update(|gi| {
            gi.show_replay = true;
        });
    };

    view! {
        <GameWidgets>
            <InactiveMines num_mines />
            <InactiveGameStateWidget game_state on_click=remake_game />
            <InactiveTimer game_time />
        </GameWidgets>
        <InactiveGame board />
        <div class="flex justify-center space-x-4 mb-6">
            <button
                class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 dark:bg-blue-600 dark:hover:bg-blue-700 cursor-pointer"
                on:click=open_replay
                title="Open Replay"
            >
                "Replay"
            </button>
        </div>
    }
}

#[component]
pub fn TauriReplayGame(
    game_data: GameData,
    set_game_signal: WriteSignal<GameData>,
) -> impl IntoView {
    let game_info = game_data.game_info.clone();
    let completed_game = game_data
        .completed_game
        .as_ref()
        .expect("Completed game should exist when showing replay");

    let game_time = game_time_from_start_end(game_info.start_time, game_info.end_time);
    let (_flag_count, set_flag_count) = signal(0);

    let board = game_info.board();

    // Create cell signals for replay
    let (cell_read_signals, cell_write_signals) = board
        .rows_iter()
        .map(|col| {
            col.iter()
                .copied()
                .map(|pc| signal(ReplayAnalysisCell(pc, None::<AnalyzedCell>)))
                .collect::<(Vec<_>, Vec<_>)>()
        })
        .collect::<(Vec<Vec<_>>, Vec<Vec<_>>)>();

    // Clone for ReplayGame before converting to Arc
    let cell_read_signals_clone = cell_read_signals.clone();

    // Create replay with analysis for single player
    let replay = completed_game
        .replay(Some(0)) // Single player is always player 0
        .expect("Replay should be available for completed game")
        .with_analysis();

    let close_replay = move |_| {
        set_game_signal.update(|gi| {
            gi.show_replay = false;
        });
    };

    view! {
        <GameWidgets>
            <InactiveMines num_mines=game_info.num_mines />
            <button
                class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 dark:bg-blue-600 dark:hover:bg-blue-700 cursor-pointer"
                on:click=close_replay
                title="Back to Game"
            >
                "Back"
            </button>
            <InactiveTimer game_time />
        </GameWidgets>
        <ReplayGame cell_read_signals=cell_read_signals_clone />
        <ReplayControls
            replay
            cell_read_signals=Arc::new(cell_read_signals)
            cell_write_signals=Arc::new(cell_write_signals)
            flag_count_setter=Some(set_flag_count)
            player_write_signals=None
        />
    }
}

