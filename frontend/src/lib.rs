use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use anyhow::Result;
use leptos::{leptos_dom::console_log, *};
use leptos_router::*;
use leptos_use::{use_websocket, UseWebSocketReadyState, UseWebsocketReturn};
use minesweeper::{
    board::BoardPoint,
    cell::PlayerCell,
    client::{MinesweeperClient, Play},
    game::Action as PlayAction,
    GameMessage,
};
use web_sys::{MouseEvent, WebSocket};

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    view! { cx,
        <div id="root">
            <Router>
                <h1>Minesweeper</h1>
                <A href="">Home</A>
            <main>
            <Routes>
                // TODO - new game & join game suspense
                <Route path="" view=|cx| view!{cx, <A href="jFSUQSLk">Start game</A>} />
                <Route path="/:id" view=|cx| view!{ cx,
                    <Game rows=16 cols=30 />
                } />
            </Routes>
            </main>
            </Router>
        </div>
    }
}

struct FrontendGame {
    cell_signals: Vec<Vec<WriteSignal<PlayerCell>>>,
    err_signal: WriteSignal<Option<String>>,
    game: Box<MinesweeperClient>,
    ws: Option<WebSocket>,
}

impl FrontendGame {
    fn try_reveal(&self, row: usize, col: usize) -> Result<()> {
        // TODO - actual player, flag, and double-click
        let play_json = serde_json::to_string(&Play {
            player: 0,
            action: PlayAction::Reveal,
            point: BoardPoint { row, col },
        })?;
        self.send(play_json);
        Ok(())
    }

    fn try_flag(&self, row: usize, col: usize) -> Result<()> {
        // TODO - actual player, flag, and double-click
        let play_json = serde_json::to_string(&Play {
            player: 0,
            action: PlayAction::Flag,
            point: BoardPoint { row, col },
        })?;
        self.send(play_json);
        Ok(())
    }

    fn try_reveal_adjacent(&self, row: usize, col: usize) -> Result<()> {
        // TODO - actual player, flag, and double-click
        let play_json = serde_json::to_string(&Play {
            player: 0,
            action: PlayAction::RevealAdjacent,
            point: BoardPoint { row, col },
        })?;
        self.send(play_json);
        Ok(())
    }

    fn handle_message(&mut self, msg: &str) -> Result<()> {
        console_log(msg);
        let game_message: GameMessage = serde_json::from_str(msg)?;
        console_log(&format!("{:?}", game_message));
        match game_message {
            GameMessage::PlayOutcome(po) => {
                let plays = self.game.update(po);
                plays.iter().for_each(|(point, cell)| {
                    match cell {
                        PlayerCell::Revealed(_) => self.update_cell(*point, *cell),
                        PlayerCell::Flag => self.update_cell(*point, *cell),
                        PlayerCell::Hidden => {}
                    }
                    if self.game.game_over {
                        self.close();
                    }
                });
                Ok(())
            }
            GameMessage::Error(e) => {
                (self.err_signal)(Some(e));
                Ok(())
            }
            GameMessage::GameState(gs) => {
                self.game.set_state(gs);
                self.game
                    .player_board()
                    .iter()
                    .enumerate()
                    .for_each(|(row, vec)| {
                        vec.iter().enumerate().for_each(|(col, cell)| {
                            (self.cell_signals[row][col])(*cell);
                        })
                    });
                Ok(())
            }
        }
    }

    fn update_cell(&self, point: BoardPoint, cell: PlayerCell) {
        self.cell_signals[point.row][point.col](cell);
    }

    fn send(&self, s: String) {
        if let Some(web_socket) = self.ws.borrow() {
            let _ = web_socket.send_with_str(&s);
        }
    }

    fn close(&self) {
        if let Some(web_socket) = self.ws.borrow() {
            let _ = web_socket.close();
        }
    }
}

#[component]
pub fn Game(cx: Scope, rows: usize, cols: usize) -> impl IntoView {
    let params = use_params_map(cx);
    let game_id = move || params.with(|params| params.get("id").cloned().unwrap_or_default());

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
    let (error, set_error) = create_signal::<Option<String>>(cx, None);

    // TODO - use_websocket causes panic on route change - investigate
    let UseWebsocketReturn {
        ready_state,
        message,
        ws,
        ..
    } = use_websocket(cx, "ws://127.0.0.1:3000/api/websocket".to_string());
    let ws = match ws {
        None => None,
        Some(websocket) => Some(websocket.clone()),
    };

    let game = Rc::new(RefCell::new(FrontendGame {
        cell_signals: write_signals,
        err_signal: set_error,
        game: Box::new(game),
        ws,
    }));

    provide_context(cx, Rc::clone(&game));

    let game_clone = Rc::clone(&game);
    create_effect(cx, move |_| {
        if ready_state() == UseWebSocketReadyState::Open {
            let game = (*game_clone).borrow();
            game.send(game_id());
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
        <div>{
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
        <div>{
            cells
                .into_iter()
                .enumerate()
                .map(move |(col, cell)| view! {cx, <Cell row=row col=col cell=cell /> })
                .collect_view(cx)
        }</div>
    }
}

#[component]
fn Cell(cx: Scope, row: usize, col: usize, cell: ReadSignal<PlayerCell>) -> impl IntoView {
    let id = format!("{}_{}", row, col);
    let (skip_mouseup, set_skip_mouseup) = create_signal(cx, 0);

    let handle_action = move |pa: PlayAction| {
        let game = use_context::<Rc<RefCell<FrontendGame>>>(cx).unwrap();
        let game = (*game).borrow();
        let res = match pa {
            PlayAction::Reveal => game.try_reveal(row, col),
            PlayAction::Flag => game.try_flag(row, col),
            PlayAction::RevealAdjacent => game.try_reveal_adjacent(row, col),
        };
        res.unwrap_or_else(|e| (game.err_signal)(Some(format!("{:?}", e))));
    };
    let handle_mousedown = move |ev: MouseEvent| {
        if ev.buttons() == 3 {
            set_skip_mouseup(2);
            handle_action(PlayAction::RevealAdjacent);
        }
    };
    let handle_mouseup = move |ev: MouseEvent| {
        if skip_mouseup() > 0 {
            set_skip_mouseup(skip_mouseup() - 1);
            return;
        }
        if ev.button() == 0 {
            handle_action(PlayAction::Reveal);
        }
        if ev.button() == 2 {
            handle_action(PlayAction::Flag);
        }
    };

    view! { cx,
        <span
        class="cell"
        id=id
        on:mouseup=handle_mouseup
        on:mousedown=handle_mousedown
        oncontextmenu="event.preventDefault();" >
            {move || format!("{:?}", cell()) }
        </span>
    }
}
