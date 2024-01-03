mod cell;
mod client;
pub mod players;

use anyhow::Result;
use leptos::*;
use leptos_router::*;
use leptos_use::{core::ConnectionReadyState, use_websocket, UseWebsocketReturn};
use std::{cell::RefCell, rc::Rc};

use minesweeper::{
    cell::PlayerCell,
    client::{ClientPlayer, MinesweeperClient},
};

use cell::Cell;
use client::FrontendGame;

#[cfg(feature = "ssr")]
use crate::backend::users::AuthSession;

#[component]
pub fn Game(rows: usize, cols: usize) -> impl IntoView {
    // TODO - game_id should be parameter, and there should be a parent component that renders
    // Games based on id param (with Suspense)
    let params = use_params_map();
    let game_id = move || params.get().get("id").cloned().unwrap_or_default();

    let game = MinesweeperClient::new(rows, cols);
    let curr_board = game.player_board();
    let mut read_signals: Vec<Vec<ReadSignal<PlayerCell>>> = Vec::new();
    let mut write_signals: Vec<Vec<WriteSignal<PlayerCell>>> = Vec::new();
    curr_board.iter().for_each(|v| {
        let mut read_row = Vec::new();
        let mut write_row = Vec::new();
        v.iter().for_each(|c| {
            let (rs, ws) = create_signal(*c);
            read_row.push(rs);
            write_row.push(ws);
        });
        read_signals.push(read_row);
        write_signals.push(write_row);
    });
    let mut players: Vec<ReadSignal<Option<ClientPlayer>>> = Vec::new();
    let mut player_signals: Vec<WriteSignal<Option<ClientPlayer>>> = Vec::new();
    game.players.iter().for_each(|_| {
        let (rs, ws) = create_signal(None);
        players.push(rs);
        player_signals.push(ws);
    });
    let (player, set_player) = create_signal::<Option<usize>>(None);
    let (error, set_error) = create_signal::<Option<String>>(None);
    let (skip_mouseup, set_skip_mouseup) = create_signal::<usize>(0);

    let UseWebsocketReturn {
        ready_state,
        message,
        ws,
        ..
    } = use_websocket("ws://localhost:3000/api/websocket");
    let ws = ws.clone();

    let game = Rc::new(RefCell::new(FrontendGame {
        game_id: game_id.into_signal(),
        cell_signals: write_signals,
        player,
        set_player,
        players,
        player_signals,
        skip_mouseup,
        set_skip_mouseup,
        err_signal: set_error,
        game: Box::new(game),
        ws,
    }));

    provide_context(Rc::clone(&game));

    let game_clone = Rc::clone(&game);
    create_effect(move |_| {
        if ready_state() == ConnectionReadyState::Open {
            let game = (*game_clone).borrow();
            game.send(game_id());
        }
    });

    let game_clone = Rc::clone(&game);
    create_effect(move |_| {
        if let Some(msg) = message() {
            let mut game = (*game_clone).borrow_mut();
            let res = game.handle_message(&msg);
            if let Err(e) = res {
                (game.err_signal)(Some(format!("{:?}", e)))
            } else {
                (game.err_signal)(None)
            }
        }
    });

    view! {
        <Outlet/>
        <div class="board">
            {read_signals
                .into_iter()
                .enumerate()
                .map(move |(row, vec)| view! { <Row row=row cells=vec/> })
                .collect_view()}
        </div>
        <div class="error">{error}</div>
    }
}

#[component]
fn Row(row: usize, cells: Vec<ReadSignal<PlayerCell>>) -> impl IntoView {
    view! {
        <div class="row">
            {cells
                .into_iter()
                .enumerate()
                .map(move |(col, cell)| view! { <Cell row=row col=col cell=cell/> })
                .collect_view()}
        </div>
    }
}

#[server(StartGame, "/api")]
async fn start_game() -> Result<(), ServerFnError> {
    let auth_session = use_context::<AuthSession>()
        .ok_or_else(|| ServerFnError::ServerError("Unable to find auth session".to_string()))?;

    let _user = match auth_session.user {
        Some(user) => user,
        None => {
            return Err(ServerFnError::ServerError("Not logged in".to_string()));
        }
    };

    todo!()
    // let id = game_manager.create_game(user);
    // leptos_axum::redirect(&format!("/game/{}", id));
    // Ok(())
}

#[component]
pub fn StartGame() -> impl IntoView {
    let new_game = create_server_action::<StartGame>();

    view! {
        <ActionForm action=new_game >

            <button type="submit">"New Game"</button>
        </ActionForm>
    }
}
