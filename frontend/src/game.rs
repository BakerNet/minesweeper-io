mod cell;
mod client;
pub mod players;

use cell::Cell;
use client::FrontendGame;

use std::{cell::RefCell, rc::Rc};

use leptos::*;
use leptos_router::*;
use leptos_use::{use_websocket, UseWebSocketReadyState, UseWebsocketReturn};
use minesweeper::{
    cell::PlayerCell,
    client::{ClientPlayer, MinesweeperClient},
};

#[component]
pub fn Game(cx: Scope, rows: usize, cols: usize) -> impl IntoView {
    let params = use_params_map(cx);
    let game_id = params.get().get("id").cloned().unwrap_or_default();

    let game = MinesweeperClient::new(rows, cols);
    let curr_board = game.player_board();
    let mut read_signals: Vec<Vec<ReadSignal<PlayerCell>>> = Vec::new();
    let mut write_signals: Vec<Vec<WriteSignal<PlayerCell>>> = Vec::new();
    curr_board.iter().for_each(|v| {
        let mut read_row = Vec::new();
        let mut write_row = Vec::new();
        v.iter().for_each(|c| {
            let (rs, ws) = create_signal(cx, *c);
            read_row.push(rs);
            write_row.push(ws);
        });
        read_signals.push(read_row);
        write_signals.push(write_row);
    });
    let mut players: Vec<ReadSignal<Option<ClientPlayer>>> = Vec::new();
    let mut player_signals: Vec<WriteSignal<Option<ClientPlayer>>> = Vec::new();
    game.players.iter().for_each(|_| {
        let (rs, ws) = create_signal(cx, None);
        players.push(rs);
        player_signals.push(ws);
    });
    let (player, set_player) = create_signal::<Option<usize>>(cx, None);
    let (error, set_error) = create_signal::<Option<String>>(cx, None);
    let (skip_mouseup, set_skip_mouseup) = create_signal::<usize>(cx, 0);

    // TODO - use_websocjet causes panic on route change - investigate
    let UseWebsocketReturn {
        ready_state,
        message,
        ws,
        ..
    } = use_websocket(cx, "ws://127.0.0.1:3000/api/websocket".to_string());
    let ws = ws.clone();

    let game = Rc::new(RefCell::new(FrontendGame {
        game_id: game_id.clone(),
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

    provide_context(cx, Rc::clone(&game));

    let game_clone = Rc::clone(&game);
    create_effect(cx, move |_| {
        if ready_state() == UseWebSocketReadyState::Open {
            let game = (*game_clone).borrow();
            game.send(game_id.clone());
        }
    });

    let game_clone = Rc::clone(&game);
    create_effect(cx, move |_| {
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

    view! { cx,
        <Outlet />
        <div class="board">{
            read_signals
                .into_iter()
                .enumerate()
                .map(move |(row, vec)| view!{cx, <Row row=row cells=vec />})
                .collect_view(cx)
        }</div>
        <div class="error">{error}</div>
    }
}

#[component]
fn Row(cx: Scope, row: usize, cells: Vec<ReadSignal<PlayerCell>>) -> impl IntoView {
    view! { cx,
        <div class="row" >{
            cells
                .into_iter()
                .enumerate()
                .map(move |(col, cell)| view! {cx, <Cell row=row col=col cell=cell /> })
                .collect_view(cx)
        }</div>
    }
}
