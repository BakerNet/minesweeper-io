use crate::game::{FrontendGame, GameModeStats, SavedGame};
use crate::GameData;
use chrono::{DateTime, Utc};
use game_ui::{
    button_class, game_time_from_start_end,
    icons::{Mine, Trophy},
    parse_timeline_stats, player_icon_holder, ActiveGame, ActiveMines, ActiveTimer, GameInfo,
    GameMode, GameSettings, GameState, GameStateWidget, GameWidgets, InactiveGame,
    InactiveGameStateWidget, InactiveMines, InactiveTimer, PlayerGameModeStats, PlayerStats,
    PlayerStatsRow, PlayerStatsTable, ReplayControls, ReplayGame, TimelineStats,
    TimelineStatsGraphs,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos::task::spawn_local;
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
                    class=button_class!("", "bg-sky-700 text-white hover:bg-sky-900/90")
                    on:click=move |_| {
                        new_game(GameMode::ClassicBeginner.into());
                        set_show_custom(false);
                    }
                    title="Start Beginner Game"
                >
                    "Beginner"
                </button>
                <button
                    class=button_class!("", "bg-green-700 text-white hover:bg-green-800/90")
                    on:click=move |_| {
                        new_game(GameMode::ClassicIntermediate.into());
                        set_show_custom(false);
                    }
                    title="Start Intermediate Game"
                >
                    "Intermediate"
                </button>
                <button
                    class=button_class!("", "bg-red-600 text-white hover:bg-red-700/90")
                    on:click=move |_| {
                        new_game(GameMode::ClassicExpert.into());
                        set_show_custom(false);
                    }
                    title="Start Expert Game"
                >
                    "Expert"
                </button>
                <button
                    class=button_class!("", "bg-purple-700 text-white hover:bg-purple-800/90")
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
                                    set_custom_rows(
                                        event_target_value(&ev).parse::<i64>().unwrap_or(9),
                                    );
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
                                    set_custom_cols(
                                        event_target_value(&ev).parse::<i64>().unwrap_or(9),
                                    );
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
                                    set_custom_mines(
                                        event_target_value(&ev).parse::<i64>().unwrap_or(10),
                                    );
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
                            {move || {
                                custom_errors
                                    .get()
                                    .into_iter()
                                    .map(|err| view! { <div>{err}</div> })
                                    .collect_view()
                            }}
                        </div>
                    </Show>
                    <button
                        class=button_class!(
                            "w-full rounded", "bg-green-700 text-white hover:bg-green-800/90"
                        )
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
        if is_completed && prev_completed != Some(true) {
            // Game just completed, extract the CompletedMinesweeper
            game.with_value(|g| {
                if let Some(completed_minesweeper) = g.extract_completed_game() {
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
                    let completed_game = Arc::new(completed_minesweeper);
                    let updated_data =
                        GameData::with_completed(new_game_info.clone(), completed_game.clone());
                    set_game_signal(updated_data);

                    // Extract signal data before spawn_local to avoid accessing dropped signals
                    let is_completed = g.completed.get_untracked();
                    let victory = g.victory.get_untracked();
                    let start_time = g.start_time.get_untracked();

                    // Automatically save the completed game
                    spawn_local(async move {
                        if let Err(e) = FrontendGame::save_game_with_completed(
                            &new_game_info,
                            is_completed,
                            victory,
                            start_time,
                            Some(completed_game),
                        )
                        .await
                        {
                            log::error!("Failed to auto-save game: {}", e);
                        } else {
                            log::info!("Game automatically saved");
                        }
                    });
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
    game_data: GameData,
    set_game_signal: WriteSignal<GameData>,
) -> impl IntoView {
    let game_info = &game_data.game_info;
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

    let game_data = StoredValue::new(game_data);

    // Count mines in the final board
    let num_mines = board
        .rows_iter()
        .flatten()
        .filter(|&c| matches!(c, PlayerCell::Hidden(HiddenCell::Mine)))
        .count();

    let remake_game = move || {
        let game_data = game_data.get_value();
        set_game_signal.set(GameData::new(GameInfo::new_singleplayer(
            String::new(),
            game_data.game_info.rows,
            game_data.game_info.cols,
            game_data.game_info.num_mines,
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
                class=button_class!("rounded", "bg-green-700 text-white hover:bg-green-800/90")
                on:click=move |_| remake_game()
                title="Play Again"
            >
                "Play Again"
            </button>
            <button
                class=button_class!("rounded", "bg-blue-600 text-white hover:bg-blue-700/90")
                on:click=open_replay
                title="Open Replay"
            >
                "Open Replay"
            </button>
        </div>
    }
}

#[component]
pub fn TauriReplayGame(
    game_data: GameData,
    set_game_signal: WriteSignal<GameData>,
) -> impl IntoView {
    let game_info = game_data.game_info;
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
                class=button_class!("rounded", "bg-neutral-700 text-white hover:bg-neutral-800/90")
                on:click=close_replay
                title="Back to Game"
            >
                "Back"
            </button>
            <InactiveTimer game_time />
        </GameWidgets>
        <ReplayGame cell_read_signals />
        <ReplayControls
            replay
            cell_read_signals=Arc::new(cell_read_signals_clone)
            cell_write_signals=Arc::new(cell_write_signals)
            flag_count_setter=Some(set_flag_count)
            player_write_signals=None
        />
    }
}

fn convert_game_mode_stats(stats: &GameModeStats) -> PlayerGameModeStats {
    PlayerGameModeStats {
        played: stats.played as usize,
        victories: stats.victories as usize,
        best_time: stats.best_time.map(|t| t as usize).unwrap_or(0),
        average_time: stats.average_time.unwrap_or(0.0),
    }
}

fn convert_timeline_data(timeline_data: &[crate::game::TimelineGameData]) -> Vec<(bool, i64)> {
    let result: Vec<(bool, i64)> = timeline_data
        .iter()
        .map(|data| (data.victory, data.seconds as i64))
        .collect();

    log::info!("Timeline data converted: {:?}", result);
    result
}

#[component]
pub fn GameStatsModal(set_show_stats: WriteSignal<bool>) -> impl IntoView {
    let player_stats = LocalResource::new(move || async move {
        let aggregate_stats = FrontendGame::get_aggregate_stats().await;
        aggregate_stats.ok().map(|stats| PlayerStats {
            beginner: convert_game_mode_stats(&stats.beginner),
            intermediate: convert_game_mode_stats(&stats.intermediate),
            expert: convert_game_mode_stats(&stats.expert),
        })
    });
    let timeline_stats = LocalResource::new(move || async move {
        let tl_stats = FrontendGame::get_timeline_stats().await;
        tl_stats.ok().map(|stats| TimelineStats {
            beginner: parse_timeline_stats(&convert_timeline_data(&stats.beginner)),
            intermediate: parse_timeline_stats(&convert_timeline_data(&stats.intermediate)),
            expert: parse_timeline_stats(&convert_timeline_data(&stats.expert)),
        })
    });

    view! {
        <div class="p-8 max-w-6xl mx-auto">
            <div class="flex justify-between items-center mb-6">
                <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-200">
                    "Game Statistics"
                </h2>
                <button
                    class=button_class!(
                        "p-2 rounded-full transition-colors", "bg-transparent hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-800 dark:text-gray-200"
                    )
                    on:click=move |_| set_show_stats(false)
                    title="Close"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-6 w-6"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M6 18L18 6M6 6l12 12"
                        />
                    </svg>
                </button>
            </div>

            <Suspense fallback=move || {
                view! {
                    <div class="flex justify-center items-center py-8">
                        <div class="text-gray-600 dark:text-gray-400">Loading stats...</div>
                    </div>
                }
            }>
                <div class="flex flex-col items-center">
                    {move || {
                        if let Some(stats) = player_stats.get().flatten() {
                            Either::Left(
                                view! {
                                    <div class="flex flex-col items-center">
                                        <PlayerStatsTable>
                                            <PlayerStatsRow
                                                mode=GameMode::ClassicBeginner
                                                stats=stats.beginner
                                            />
                                            <PlayerStatsRow
                                                mode=GameMode::ClassicIntermediate
                                                stats=stats.intermediate
                                            />
                                            <PlayerStatsRow
                                                mode=GameMode::ClassicExpert
                                                stats=stats.expert
                                            />
                                        </PlayerStatsTable>
                                        <h2 class="text-2xl my-4 text-gray-900 dark:text-gray-200 text-center">
                                            "Performance Over Time"
                                        </h2>
                                        <div class="flex justify-center w-full">
                                            <TimelineStatsGraphs timeline_stats=Signal::derive(move || timeline_stats.get().flatten()) />
                                        </div>
                                    </div>
                                },
                            )
                        } else {
                            Either::Right(
                                view! {
                                    <div class="text-center py-8">
                                        <p class="text-gray-600 dark:text-gray-400">
                                            "No game data available. Play some games to see your statistics!"
                                        </p>
                                    </div>
                                },
                            )
                        }
                    }}
                </div>
            </Suspense>
        </div>
    }
}

#[component]
pub fn SavedGamesList(
    set_game_signal: WriteSignal<GameData>,
    set_show_saved_games: WriteSignal<bool>,
) -> impl IntoView {
    let saved_games =
        LocalResource::new(move || async move { FrontendGame::get_saved_games().await.ok() });

    let load_replay = Callback::new(move |game: SavedGame| {
        set_show_saved_games(false); // Close the modal after loading
        spawn_local(async move {
            match FrontendGame::reconstruct_completed_game(&game) {
                Ok(Some(completed_game)) => {
                    let mut game_info = GameInfo::new_singleplayer(
                        game.game_id.clone(),
                        game.rows as usize,
                        game.cols as usize,
                        game.num_mines as usize,
                    );
                    game_info.is_completed = game.is_completed;
                    game_info.start_time = game.start_time.as_ref().and_then(|s| s.parse().ok());
                    game_info.end_time = game.end_time.as_ref().and_then(|s| s.parse().ok());
                    if let Some(board) = game.final_board {
                        if let Ok(board) = serde_json::from_str(&board) {
                            game_info.final_board = board;
                        }
                    }

                    let mut game_data =
                        GameData::with_completed(game_info, Arc::new(completed_game));
                    game_data.show_replay = true; // Automatically show replay when loading from saved games
                    set_game_signal(game_data);
                }
                Ok(None) => {
                    // Could use a toast or notification here
                    log::error!("No replay data available for this game");
                }
                Err(e) => {
                    // Could use a toast or notification here
                    log::error!("Failed to load replay: {}", e);
                }
            }
        });
    });

    view! {
        <div class="p-8">
            <div class="flex justify-between items-center mb-6">
                <h2 class="text-2xl font-bold text-gray-800 dark:text-gray-200">"Saved Games"</h2>
                <button
                    class=button_class!(
                        "p-2 rounded-full transition-colors", "bg-transparent hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-800 dark:text-gray-200"
                    )
                    on:click=move |_| set_show_saved_games(false)
                    title="Close"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-6 w-6"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M6 18L18 6M6 6l12 12"
                        />
                    </svg>
                </button>
            </div>

            <Suspense fallback=move || {
                view! {
                    <div class="flex justify-center items-center py-8">
                        <div class="text-gray-600 dark:text-gray-400">Loading saved games...</div>
                    </div>
                }
            }>
                {move || {
                    saved_games
                        .get()
                        .flatten()
                        .map(|games| {
                            view! {
                                <div class="grid gap-6">
                                    {games.iter().map(move |game| view!{ <SavedGameRow game=game.to_owned() load_replay /> }).collect_view()}
                                    </div>
                                    <Show when=move || games.is_empty()>
                                        <div class="text-center py-8">
                                            <p class="text-gray-600 dark:text-gray-400"> "No saved games found. Complete some games to see them here!" </p>
                                        </div>
                                    </Show>
                                }
                            })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn SavedGameRow(game: SavedGame, load_replay: Callback<SavedGame>) -> impl IntoView {
    let has_replay = game.game_log.is_some() && game.final_board.is_some();
    let game_duration = match (&game.start_time, &game.end_time) {
        (Some(start), Some(end)) => {
            match (start.parse::<DateTime<Utc>>(), end.parse::<DateTime<Utc>>()) {
                (Ok(start_dt), Ok(end_dt)) => {
                    let duration = end_dt - start_dt;
                    Some(duration.num_seconds())
                }
                _ => None,
            }
        }
        _ => None,
    };
    let formatted_start_time = game.start_time.as_ref().and_then(|t| {
        t.parse::<DateTime<Utc>>()
            .ok()
            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
    });
    let game_mode_display = match (game.rows, game.cols, game.num_mines) {
        (9, 9, 10) => "Beginner".to_string(),
        (16, 16, 40) => "Intermediate".to_string(),
        (16, 30, 99) => "Expert".to_string(),
        _ => format!("Custom({}x{})", game.rows, game.cols),
    };
    let result_icon = if game.victory {
        // Calculate game duration in seconds

        // Format start time for display

        // Determine game mode

        // Victory/defeat icon
        view! {
            <span class=player_icon_holder!("bg-green-800")>
                <Trophy />
            </span>
        }
        .into_any()
    } else {
        view! {
            <span class=player_icon_holder!("bg-red-600")>
                <Mine />
            </span>
        }
        .into_any()
    };

    let game_stored = StoredValue::new(game);

    view! {
        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-4">
            <div class="flex justify-between items-center mb-2">
                <div class="flex items-center mr-4">
                    <h3 class="text-lg font-semibold text-gray-800 dark:text-gray-200 mr-4">
                        {game_mode_display}
                    </h3>
                    {result_icon}
                    <div class="flex flex-col items-center justify-center border-2 border-slate-400 bg-neutral-200 text-neutral-800 text-sm font-bold px-1 py-0.5 ml-4 h-5 w-auto min-w-[2rem]">
                        {game_duration
                            .map(|d| format!("{}s", d))
                            .unwrap_or_else(|| "?".to_string())}
                    </div>
                </div>
                <div class="flex-shrink-0">
                    <Show when=move || has_replay>
                        <button
                            class=button_class!(
                                "px-3 py-1 text-sm rounded", "bg-green-700 text-white hover:bg-green-800/90"
                            )
                            on:click=move |_| load_replay.run(game_stored.get_value())
                        >
                            "Load Replay"
                        </button>
                    </Show>
                    <Show when=move || !has_replay>
                        <span class="px-3 py-1 bg-gray-300 text-gray-500 rounded text-sm">
                            "No Replay"
                        </span>
                    </Show>
                </div>
            </div>
            <div class="text-sm text-gray-600 dark:text-gray-400">
                {formatted_start_time
                    .map(|t| format!("Played: {}", t))
                    .unwrap_or_else(|| "Date: Unknown".to_string())}
            </div>
        </div>
    }
}
