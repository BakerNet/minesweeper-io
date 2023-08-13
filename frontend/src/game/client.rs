use std::borrow::Borrow;

use anyhow::{anyhow, Result};
use leptos::{leptos_dom::console_log, *};
use minesweeper::{
    board::BoardPoint,
    cell::PlayerCell,
    client::{MinesweeperClient, Play},
    game::Action as PlayAction,
    GameMessage,
};
use web_sys::WebSocket;

pub struct FrontendGame {
    pub cell_signals: Vec<Vec<WriteSignal<PlayerCell>>>,
    pub skip_mouseup: ReadSignal<usize>,
    pub set_skip_mouseup: WriteSignal<usize>,
    pub err_signal: WriteSignal<Option<String>>,
    pub game: Box<MinesweeperClient>,
    pub ws: Option<WebSocket>,
}

impl FrontendGame {
    pub fn try_reveal(&self, row: usize, col: usize) -> Result<()> {
        // TODO - actual player, flag, and double-click
        let play_json = serde_json::to_string(&Play {
            player: 0,
            action: PlayAction::Reveal,
            point: BoardPoint { row, col },
        })?;
        self.send(play_json);
        Ok(())
    }

    pub fn try_flag(&self, row: usize, col: usize) -> Result<()> {
        // TODO - actual player, flag, and double-click
        let play_json = serde_json::to_string(&Play {
            player: 0,
            action: PlayAction::Flag,
            point: BoardPoint { row, col },
        })?;
        self.send(play_json);
        Ok(())
    }

    pub fn try_reveal_adjacent(&self, row: usize, col: usize) -> Result<()> {
        // TODO - actual player, flag, and double-click
        let play_json = serde_json::to_string(&Play {
            player: 0,
            action: PlayAction::RevealAdjacent,
            point: BoardPoint { row, col },
        })?;
        self.send(play_json);
        Ok(())
    }

    pub fn handle_message(&mut self, msg: &str) -> Result<()> {
        console_log(msg);
        let game_message: GameMessage = serde_json::from_str(msg)?;
        console_log(&format!("{:?}", game_message));
        match game_message {
            GameMessage::PlayOutcome(po) => {
                let plays = self.game.update(po);
                plays.iter().for_each(|(point, cell)| {
                    console_log(&format!("{:?} {:?}", point, cell));
                    self.update_cell(*point, *cell);
                    if self.game.game_over {
                        self.close();
                    }
                });
                Ok(())
            }
            GameMessage::Error(e) => Err(anyhow!(e)),
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

    pub fn update_cell(&self, point: BoardPoint, cell: PlayerCell) {
        self.cell_signals[point.row][point.col](cell);
    }

    pub fn send(&self, s: String) {
        if let Some(web_socket) = self.ws.borrow() {
            let _ = web_socket.send_with_str(&s);
        }
    }

    pub fn close(&self) {
        if let Some(web_socket) = self.ws.borrow() {
            let _ = web_socket.close();
        }
    }
}
