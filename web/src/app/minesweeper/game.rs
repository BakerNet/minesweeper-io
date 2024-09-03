use chrono::DateTime;
use codee::string::JsonSerdeWasmCodec;
use leptos::{either::*, ev, prelude::*};
use leptos_router::{components::*, hooks::*};
use leptos_use::{
    core::ConnectionReadyState, use_document, use_event_listener, use_websocket, UseWebSocketReturn,
};
use std::sync::Arc;
use web_sys::{KeyboardEvent, MouseEvent};

use minesweeper_lib::{
    board::{Board, BoardPoint},
    cell::{HiddenCell, PlayerCell},
    game::{Action as PlayAction, CompletedMinesweeper},
};

use super::{
    cell::{ActiveCell, InactiveCell, ReplayCell},
    client::{signals_from_board, FrontendGame, PlayersContext},
    entry::ReCreateGame,
    players::{ActivePlayers, InactivePlayers, PlayerButtons},
    replay::{OpenReplay, ReplayControls},
    widgets::{ActiveMines, ActiveTimer, CopyGameLink, GameWidgets, InactiveMines, InactiveTimer},
    {GameInfo, GameInfoWithLog, GameSettings},
};

#[cfg(feature = "ssr")]
use crate::backend::{AuthSession, GameManager};
use crate::messages::{ClientMessage, GameMessage};
#[cfg(feature = "ssr")]
use minesweeper_lib::client::ClientPlayer;

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

    log::debug!("one");
    let is_owner = if let Some(user) = &auth_session.user {
        match game.owner {
            None => false,
            Some(owner) => user.id == owner,
        }
    } else {
        false
    };
    log::debug!("two");
    let user_ref = auth_session.user.as_ref();
    let player_num = if user_ref.is_some() {
        players
            .iter()
            .find(|p| p.user == user_ref.map(|u| u.id))
            .map(|p| p.player)
    } else {
        None
    };
    log::debug!("three");
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
        (fb, _) => fb.unwrap_or(vec![
            vec![PlayerCell::default(); game.cols as usize];
            game.rows as usize
        ]),
    };
    log::debug!("four");
    let players_frontend =
        players
            .into_iter()
            .fold(vec![None; game.max_players as usize], |mut acc, p| {
                let index = p.player as usize;
                acc[index] = Some(ClientPlayer::from(p));
                acc
            });
    log::debug!("five");
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
        final_board,
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
    let player_num = if user_ref.is_some() {
        players
            .iter()
            .find(|p| p.user == user_ref.map(|u| u.id))
            .map(|p| p.player)
    } else if game.max_players == 1 {
        Some(0)
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
    let log = completed_minesweeper.recover_log().unwrap();
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
            final_board,
            players: players_frontend,
        },
        player_num,
        log,
    })
}

#[component]
pub fn GameWrapper() -> impl IntoView {
    let params = use_params_map();

    Effect::new(move |_| {
        log::debug!("Outer Params: {:?}", params.get());
    });

    let game_id = move || params.get().get("id").unwrap_or_default();

    view! {
        <div class="text-center">
            <h3 class="text-4xl my-4 text-gray-900 dark:text-gray-200">"Game: "{game_id}</h3>
            <Outlet />
        </div>
    }
}

#[component]
pub fn GameView() -> impl IntoView {
    let params = use_params_map();
    let game_id = move || params.get().get("id").unwrap_or_default();
    let game_info = Resource::new(game_id, get_game);
    let refetch = move || game_info.refetch();

    Effect::new(move |_| {
        log::debug!("Inner Params: {:?}", params.get());
    });

    let game_view = move |game_info: GameInfo| match game_info.is_completed {
        true => Either::Left(view! { <InactiveGame game_info /> }),
        false => Either::Right(view! { <ActiveGame game_info refetch /> }),
    };

    view! {
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
pub fn GameReplay() -> impl IntoView {
    let params = use_params_map();
    let game_id = move || params.get().get("id").unwrap_or_default();
    let game_info = Resource::new(game_id, get_replay);

    let game_view = move |replay_data: GameInfoWithLog| match replay_data.game_info.is_completed {
        true => Either::Left(view! { <ReplayGame replay_data /> }),
        false => Either::Right(
            view! { <Redirect path=format!("/game/{}", replay_data.game_info.game_id) /> },
        ),
    };

    view! {
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
fn GameBorder<F>(set_active: F, children: Children) -> impl IntoView
where
    F: Fn(bool) + Copy + 'static,
{
    view! {
        <div class="select-none overflow-x-auto overflow-y-hidden mb-8">
            <div class="w-fit border-solid border border-black mx-auto">
                <div
                    class="w-fit border-groove border-24"
                    on:mouseenter=move |_| set_active(true)
                    on:mouseleave=move |_| set_active(false)
                >
                    {children()}
                </div>
            </div>
        </div>
    }
}

#[component]
fn ActiveGame<F>(game_info: GameInfo, refetch: F) -> impl IntoView
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
    let players_context = PlayersContext::from(&game);
    let flag_count = game.flag_count;
    let completed = game.completed;
    let sync_time = game.sync_time;
    let cells = Arc::clone(&game.cells);
    let players = Arc::clone(&game.players);

    let (game_signal, _) = signal_local(game);

    Effect::watch(
        ready_state,
        move |state, _, _| {
            log::debug!("before ready_state");
            let game = game_signal.get_untracked();
            match state {
                ConnectionReadyState::Open => {
                    log::debug!("ready_state Open");
                    game.send(ClientMessage::Join);
                }
                ConnectionReadyState::Closed => {
                    log::debug!("ready_state Closed");
                    refetch();
                }
                _ => {}
            }
        },
        true,
    );

    Effect::watch(
        message,
        move |msg, _, _| {
            log::debug!("before message");
            let game = game_signal.get_untracked();
            if let Some(msg) = msg {
                log::debug!("after message {:?}", msg);
                let res = game.handle_message(msg.clone());
                if let Err(e) = res {
                    (game.err_signal)(Some(format!("{:?}", e)));
                } else {
                    (game.err_signal)(None);
                }
            }
        },
        false,
    );

    Effect::new(move |prev: Option<bool>| {
        let game = game_signal.get_untracked();
        game.join.track();
        log::debug!("join_trigger rec: {prev:?}");
        if let Some(sent) = prev {
            if !sent {
                game.send(ClientMessage::PlayGame);
                return true;
            }
        }
        false
    });

    let (skip_mouseup, set_skip_mouseup) = signal::<usize>(0);
    let (game_is_active, set_game_is_active) = signal(false);
    let (active_cell, set_active_cell) = signal(BoardPoint { row: 0, col: 0 });

    let handle_action = move |pa: PlayAction, row: usize, col: usize| {
        let game = game_signal.get_untracked();
        let res = match pa {
            PlayAction::Reveal => game.try_reveal(row, col),
            PlayAction::Flag => game.try_flag(row, col),
            PlayAction::RevealAdjacent => game.try_reveal_adjacent(row, col),
        };
        res.unwrap_or_else(|e| (game.err_signal)(Some(format!("{:?}", e))));
    };

    let handle_keydown = move |ev: KeyboardEvent| {
        if !game_is_active.get() {
            return;
        }
        let BoardPoint { row, col } = active_cell.get();
        match ev.key().as_str() {
            " " => {
                ev.prevent_default();
                handle_action(PlayAction::Reveal, row, col);
            }
            "d" => {
                handle_action(PlayAction::RevealAdjacent, row, col);
            }
            "f" => {
                handle_action(PlayAction::Flag, row, col);
            }
            _ => {}
        }
    };
    let _ = use_event_listener(use_document(), ev::keydown, handle_keydown);

    let handle_mousedown = move |ev: MouseEvent, row: usize, col: usize| {
        let set_skip_signal = { set_skip_mouseup };
        if ev.button() == 2 {
            handle_action(PlayAction::Flag, row, col);
        }
        if ev.buttons() == 3 {
            set_skip_signal.set(2);
            handle_action(PlayAction::RevealAdjacent, row, col);
        }
    };
    let handle_mouseup = move |ev: MouseEvent, row: usize, col: usize| {
        if skip_mouseup.get() > 0 {
            set_skip_mouseup.set(skip_mouseup() - 1);
            return;
        }
        if ev.button() == 0 {
            handle_action(PlayAction::Reveal, row, col);
        }
    };

    view! {
        <ActivePlayers players title="Players">
            <PlayerButtons players_context />
        </ActivePlayers>
        <GameWidgets>
            <ActiveMines num_mines=game_info.num_mines flag_count />
            <CopyGameLink game_id=game_info.game_id />
            <ActiveTimer sync_time completed />
        </GameWidgets>
        <GameBorder set_active=set_game_is_active>
            {cells
                .iter()
                .enumerate()
                .map(move |(row, vec)| {
                    view! {
                        <div class="whitespace-nowrap">
                            {vec
                                .iter()
                                .copied()
                                .enumerate()
                                .map(move |(col, cell)| {
                                    view! {
                                        <ActiveCell
                                            row=row
                                            col=col
                                            cell=cell
                                            set_active=set_active_cell
                                            mousedown_handler=handle_mousedown
                                            mouseup_handler=handle_mouseup
                                        />
                                    }
                                })
                                .collect_view()}
                        </div>
                    }
                })
                .collect_view()}

        </GameBorder>
        <div class="text-red-600 h-8">{error}</div>
    }
}

#[component]
fn InactiveGame(game_info: GameInfo) -> impl IntoView {
    let game_settings = GameSettings::from(&game_info);
    let game_time = game_time_from_start_end(game_info.start_time, game_info.end_time);
    let num_mines = game_info
        .final_board
        .iter()
        .flatten()
        .filter(|&c| matches!(c, PlayerCell::Hidden(HiddenCell::Mine)))
        .count();
    let is_victory = game_info
        .players
        .iter()
        .filter_map(|cp| cp.as_ref())
        .any(|cp| cp.victory_click);

    view! {
        <InactivePlayers
            players=game_info.players
            title=if is_victory { "Complete" } else { "Game Over" }
        />
        <GameWidgets>
            <InactiveMines num_mines=num_mines />
            <CopyGameLink game_id=game_info.game_id />
            <InactiveTimer game_time />
        </GameWidgets>
        <GameBorder set_active=move |_| {}>
            {game_info
                .final_board
                .into_iter()
                .enumerate()
                .map(move |(row, vec)| {
                    view! {
                        <div class="whitespace-nowrap">
                            {vec
                                .iter()
                                .copied()
                                .enumerate()
                                .map(move |(col, cell)| {
                                    view! { <InactiveCell row=row col=col cell=cell /> }
                                })
                                .collect_view()}
                        </div>
                    }
                })
                .collect_view()}
        </GameBorder>
        <ReCreateGame game_settings />
        <OpenReplay />
    }
}

#[component]
fn ReplayGame(replay_data: GameInfoWithLog) -> impl IntoView {
    let game_info = replay_data.game_info;
    log::debug!("replay for {:?}", game_info);
    let game_time = game_time_from_start_end(game_info.start_time, game_info.end_time);
    let (flag_count, set_flag_count) = signal(0);

    let (player_read_signals, player_write_signals) = game_info
        .players
        .iter()
        .cloned()
        .map(|p| signal(p))
        .collect::<(Vec<_>, Vec<_>)>();

    let (cell_read_signals, cell_write_signals) = signals_from_board(&game_info.final_board);
    let cell_read_signals = Arc::new(cell_read_signals);

    let completed_minesweeper = CompletedMinesweeper::from_log(
        Board::from_vec(game_info.final_board),
        replay_data.log,
        game_info.players.into_iter().flatten().collect(),
    );
    let replay = completed_minesweeper
        .replay(replay_data.player_num.map(|p| p.into()))
        .expect("We are guaranteed log is not None");

    let cells = Arc::clone(&cell_read_signals);

    view! {
        <ActivePlayers players=player_read_signals.into() title="Replay">
            {}
        </ActivePlayers>
        <GameWidgets>
            <ActiveMines num_mines=game_info.num_mines flag_count />
            <CopyGameLink game_id=game_info.game_id />
            <InactiveTimer game_time />
        </GameWidgets>
        <GameBorder set_active=move |_| ()>
            {cells
                .iter()
                .enumerate()
                .map(move |(row, vec)| {
                    view! {
                        <div class="whitespace-nowrap">
                            {vec
                                .iter()
                                .copied()
                                .enumerate()
                                .map(move |(col, cell)| {
                                    view! { <ReplayCell row=row col=col cell=cell /> }
                                })
                                .collect_view()}
                        </div>
                    }
                })
                .collect_view()}

        </GameBorder>
        <ReplayControls
            replay
            cell_read_signals
            cell_write_signals
            set_flag_count
            player_write_signals
        />
    }
}

fn game_time_from_start_end<T: chrono::TimeZone>(
    start_time: Option<DateTime<T>>,
    end_time: Option<DateTime<T>>,
) -> usize {
    (match (start_time, end_time) {
        (Some(st), Some(et)) => et.signed_duration_since(st).num_seconds(),
        _ => 999,
    }) as usize
}
