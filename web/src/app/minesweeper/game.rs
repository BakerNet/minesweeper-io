use codee::string::JsonSerdeWasmCodec;
use leptos::{either::*, prelude::*};
use leptos_meta::*;
use leptos_router::{components::*, hooks::*};
use leptos_use::{core::ConnectionReadyState, use_websocket, UseWebSocketReturn};
use std::sync::Arc;

use minesweeper_lib::{
    analysis::AnalyzedCell,
    cell::{HiddenCell, PlayerCell},
    game::{Action as PlayAction, CompletedMinesweeper},
    replay::ReplayAnalysisCell,
};

use super::{
    client::FrontendGame, entry::ReCreateGameButton, players::PlayerButtons,
    replay::OpenReplayButton,
};
use game_ui::*;

#[cfg(feature = "ssr")]
use game_manager::GameManager;
use game_manager::{ClientMessage, GameMessage};
#[cfg(feature = "ssr")]
use minesweeper_lib::{
    board::{Board, CompactBoard},
    client::ClientPlayer,
};
#[cfg(feature = "ssr")]
use web_auth::AuthSession;

#[server]
pub async fn get_game(game_id: String) -> Result<GameInfo, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    let game = game_manager
        .get_game(&game_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let players = game_manager
        .get_players(&game_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let game_log = game_manager.get_game_log(&game_id).await.ok();
    // we have all the data we need

    let is_owner = if let Some(user) = &auth_session.user {
        match game.owner {
            None => false,
            Some(owner) => user.id == owner,
        }
    } else {
        false
    };
    let user_ref = auth_session.user.as_ref();
    let player_num = if user_ref.is_some() {
        players
            .iter()
            .find(|p| p.user == user_ref.map(|u| u.id))
            .map(|p| p.player)
    } else {
        None
    };
    let players_simple = players.iter().map(ClientPlayer::from).collect::<Vec<_>>();
    let final_board = match (game.final_board, game_log) {
        (Some(board), Some(game_log)) => {
            let completed_minesweeper = CompletedMinesweeper::from_log(
                Board::from_vec(board),
                game_log.log,
                players_simple,
            );
            if let Some(p) = player_num {
                completed_minesweeper.player_board_final(p.into())
            } else if game.max_players == 1 {
                completed_minesweeper.player_board_final(0)
            } else {
                completed_minesweeper.viewer_board_final()
            }
        }
        (fb, _) => fb.map(Board::from_vec).unwrap_or(Board::new(
            game.rows as usize,
            game.cols as usize,
            PlayerCell::default(),
        )),
    };
    let players_frontend =
        players
            .into_iter()
            .fold(vec![None; game.max_players as usize], |mut acc, p| {
                let index = p.player as usize;
                acc[index] = Some(ClientPlayer::from(p));
                acc
            });
    Ok(GameInfo {
        game_id: game.game_id,
        has_owner: game.owner.is_some(),
        is_owner,
        rows: game.rows as usize,
        cols: game.cols as usize,
        num_mines: game.num_mines as usize,
        max_players: game.max_players,
        is_started: game.is_started,
        is_completed: game.is_completed,
        start_time: game.start_time,
        end_time: game.end_time,
        final_board: CompactBoard::from_board(&final_board),
        players: players_frontend,
    })
}

#[server]
pub async fn get_replay(game_id: String) -> Result<GameInfoWithLog, ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::new("Unable to find auth session".to_string()))?;
    let game_manager = use_context::<GameManager>()
        .ok_or_else(|| ServerFnError::new("No game manager".to_string()))?;
    let game = game_manager
        .get_game(&game_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    let game_log = game_manager
        .get_game_log(&game_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    if game.final_board.is_none() {
        return Err(ServerFnError::new("Game missing board data".to_string()));
    }
    let game_board = game.final_board.unwrap();
    let players = game_manager
        .get_players(&game_id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    // we have all the data we need

    let is_owner = if let Some(user) = &auth_session.user {
        match game.owner {
            None => false,
            Some(owner) => user.id == owner,
        }
    } else {
        false
    };
    let user_ref = auth_session.user.as_ref();
    let player_num = if game.max_players == 1 {
        Some(0)
    } else if user_ref.is_some() {
        players
            .iter()
            .find(|p| p.user == user_ref.map(|u| u.id))
            .map(|p| p.player)
    } else {
        None
    };
    let players_simple = players.iter().map(ClientPlayer::from).collect::<Vec<_>>();
    let completed_minesweeper =
        CompletedMinesweeper::from_log(Board::from_vec(game_board), game_log.log, players_simple);
    let final_board = if let Some(p) = player_num {
        completed_minesweeper.player_board_final(p.into())
    } else {
        completed_minesweeper.viewer_board_final()
    };
    let log = completed_minesweeper
        .recover_log()
        .unwrap()
        .into_iter()
        .map(|(play, outcome)| (play, outcome.to_compact()))
        .collect();
    let players_frontend =
        players
            .into_iter()
            .fold(vec![None; game.max_players as usize], |mut acc, p| {
                let index = p.player as usize;
                acc[index] = Some(ClientPlayer::from(p));
                acc
            });
    Ok(GameInfoWithLog {
        game_info: GameInfo {
            game_id: game.game_id,
            has_owner: game.owner.is_some(),
            is_owner,
            rows: game.rows as usize,
            cols: game.cols as usize,
            num_mines: game.num_mines as usize,
            max_players: game.max_players,
            is_started: game.is_started,
            is_completed: game.is_completed,
            start_time: game.start_time,
            end_time: game.end_time,
            final_board: CompactBoard::from_board(&final_board),
            players: players_frontend,
        },
        player_num,
        log,
    })
}

#[component]
pub fn GameWrapper() -> impl IntoView {
    let params = use_params_map();

    let game_id = move || params.get().get("id").unwrap_or_default();

    view! {
        <div class="flex-1 text-center py-8">
            <h1 class="text-4xl my-4 text-gray-900 dark:text-gray-200">"Game: "{game_id}</h1>
            <Outlet />
        </div>
    }
}

#[component]
pub fn GameView() -> impl IntoView {
    let params = use_params_map();
    let game_id = move || params.get().get("id").unwrap_or_default();
    let title = move || format!("Game {}", game_id());
    let game_info = Resource::new(game_id, get_game);
    let refetch = move || game_info.refetch();

    let game_view = move |game_info: GameInfo| match game_info.is_completed {
        true => Either::Left(view! { <WebInactiveGame game_info /> }),
        false => Either::Right(view! { <WebActiveGame game_info refetch /> }),
    };

    view! {
        <Title text=title />
        <Transition fallback=move || {
            view! { <div>"Loading..."</div> }
        }>
            {move || {
                Suspend::new(async move {
                    let game_info = game_info.await;
                    view! {
                        <ErrorBoundary fallback=|_| {
                            view! { <div class="text-red-600">"Game not found"</div> }
                        }>{game_info.map(game_view)}</ErrorBoundary>
                    }
                })
            }}

        </Transition>
    }
}

#[component]
pub fn ReplayView() -> impl IntoView {
    let params = use_params_map();
    let game_id = move || params.get().get("id").unwrap_or_default();
    let title = move || format!("Replay {}", game_id());
    let game_info = Resource::new(game_id, get_replay);

    let game_view = move |replay_data: GameInfoWithLog| match replay_data.game_info.is_completed {
        true => Either::Left(view! { <WebReplayGame replay_data /> }),
        false => Either::Right(
            view! { <Redirect path=format!("/game/{}", replay_data.game_info.game_id) /> },
        ),
    };

    view! {
        <Title text=title />
        <Transition fallback=move || {
            view! { <div>"Loading..."</div> }
        }>
            {move || {
                Suspend::new(async move {
                    let game_info = game_info.await;
                    view! {
                        <ErrorBoundary fallback=|_| {
                            view! { <div class="text-red-600">"Replay not found"</div> }
                        }>{game_info.map(game_view)}</ErrorBoundary>
                    }
                })
            }}

        </Transition>
    }
}

#[component]
fn WebActiveGame<F>(game_info: GameInfo, refetch: F) -> impl IntoView
where
    F: Fn() + Clone + 'static,
{
    let (error, set_error) = signal::<Option<String>>(None);

    let UseWebSocketReturn {
        ready_state,
        message,
        send,
        ..
    } = use_websocket::<ClientMessage, GameMessage, JsonSerdeWasmCodec>(&format!(
        "/api/websocket/game/{}",
        &game_info.game_id
    ));

    let game = FrontendGame::new(&game_info, set_error, Arc::new(send));
    let flag_count = game.flag_count;
    let completed = game.completed;
    let top_score = game.top_score;
    let sync_time = game.sync_time;
    let join_trigger = game.join_trigger;
    let players = (*game.players).clone();
    let cells = (*game.cells).clone();

    let game = StoredValue::new(game);

    Effect::watch(
        ready_state,
        move |state, _, _| {
            log::debug!("before ready_state");
            game.with_value(|game| match state {
                ConnectionReadyState::Open => {
                    log::debug!("ready_state Open");
                    game.send(ClientMessage::Join);
                }
                ConnectionReadyState::Closed => {
                    log::debug!("ready_state Closed");
                    refetch();
                }
                _ => {}
            })
        },
        true,
    );

    Effect::watch(
        message,
        move |msg, _, _| {
            log::debug!("before message");
            game.with_value(|game| {
                if let Some(msg) = msg {
                    log::debug!("after message {msg:?}");
                    let res = game.handle_message(msg.clone());
                    if let Err(e) = res {
                        (game.err_signal)(Some(format!("{e:?}")));
                    } else {
                        (game.err_signal)(None);
                    }
                }
            })
        },
        false,
    );

    Effect::new(move |prev: Option<bool>| {
        join_trigger.track();
        game.with_value(|game| {
            log::debug!("join_trigger rec: {prev:?}");
            if let Some(sent) = prev {
                if !sent {
                    game.send(ClientMessage::PlayGame);
                    return true;
                }
            }
            false
        })
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

    view! {
        <ActivePlayers players=players top_score>
            <PlayerButtons game />
        </ActivePlayers>
        <GameWidgets>
            <ActiveMines num_mines=game_info.num_mines flag_count />
            <WrappedCopyGameLink game_id=game_info.game_id />
            <ActiveTimer sync_time completed />
        </GameWidgets>
        <ActiveGame cell_read_signals=cells action_handler=handle_action />
        <div class="text-red-600 h-8">{error}</div>
    }
}

#[component]
fn WebInactiveGame(game_info: GameInfo) -> impl IntoView {
    let game_settings = GameSettings::from(&game_info);
    let game_time = game_time_from_start_end(game_info.start_time, game_info.end_time);
    let board = game_info.board();
    let num_mines = board
        .rows_iter()
        .flatten()
        .filter(|&c| matches!(c, PlayerCell::Hidden(HiddenCell::Mine)))
        .count();
    let players = game_info.players;
    let game_id = game_info.game_id;

    view! {
        <InactivePlayers players=players />
        <GameWidgets>
            <InactiveMines num_mines=num_mines />
            <WrappedCopyGameLink game_id />
            <InactiveTimer game_time />
        </GameWidgets>
        <InactiveGame board />
        <div class="flex justify-center space-x-4 mb-6">
            <ReCreateGameButton game_settings />
            <OpenReplayButton />
        </div>
    }
}

#[component]
fn WebReplayGame(replay_data: GameInfoWithLog) -> impl IntoView {
    let full_log = replay_data.full_log();
    let player_num = replay_data.player_num;
    let game_info = replay_data.game_info;
    let game_time = game_time_from_start_end(game_info.start_time, game_info.end_time);
    let (top_score, _) = signal(None);
    let (flag_count, set_flag_count) = signal(0);

    let board = game_info.board();
    let players = game_info.players;
    let game_id = game_info.game_id;
    let num_mines = game_info.num_mines;

    let (player_read_signals, player_write_signals) = players
        .iter()
        .cloned()
        .map(|p| signal(p))
        .collect::<(Vec<_>, Vec<_>)>();

    let (cell_read_signals, cell_write_signals) = board
        .rows_iter()
        .map(|col| {
            col.iter()
                .copied()
                .map(|pc| signal(ReplayAnalysisCell(pc, None::<AnalyzedCell>)))
                .collect::<(Vec<_>, Vec<_>)>()
        })
        .collect::<(Vec<Vec<_>>, Vec<Vec<_>>)>();

    let completed_minesweeper =
        CompletedMinesweeper::from_log(board, full_log, players.into_iter().flatten().collect());
    let replay = completed_minesweeper
        .replay(player_num.map(|p| p.into()))
        .expect("We are guaranteed log is not None")
        .with_analysis();

    view! {
        <ActivePlayers players=player_read_signals top_score>
            {move || {}}
        </ActivePlayers>
        <GameWidgets>
            <ActiveMines num_mines flag_count />
            <WrappedCopyGameLink game_id />
            <InactiveTimer game_time />
        </GameWidgets>
        <ReplayGame cell_read_signals=cell_read_signals.clone() />
        <ReplayControls
            replay
            cell_read_signals=Arc::new(cell_read_signals)
            cell_write_signals=Arc::new(cell_write_signals)
            flag_count_setter=Some(set_flag_count)
            player_write_signals=Some(Arc::new(player_write_signals.clone()))
        />
    }
}

#[component]
fn WrappedCopyGameLink(game_id: String) -> impl IntoView {
    #[cfg(not(feature = "ssr"))]
    let origin = { window().location().origin().unwrap_or_default() };
    #[cfg(feature = "ssr")]
    let origin = String::new();
    let game_url = format!("{origin}/game/{game_id}");
    view! { <CopyGameLink game_url /> }
}
