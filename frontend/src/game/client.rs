use std::borrow::Borrow;

use anyhow::{anyhow, bail, Result};
use leptos::{leptos_dom::console_log, *};
use minesweeper::{
    board::BoardPoint,
    cell::PlayerCell,
    client::{ClientPlayer, MinesweeperClient, Play},
    game::Action as PlayAction,
    GameMessage,
};
use web_sys::WebSocket;

pub struct FrontendGame {
    pub game_id: String,
    pub cell_signals: Vec<Vec<WriteSignal<PlayerCell>>>,
    pub player: ReadSignal<Option<usize>>,
    pub set_player: WriteSignal<Option<usize>>,
    pub players: Vec<ReadSignal<Option<ClientPlayer>>>,
    pub player_signals: Vec<WriteSignal<Option<ClientPlayer>>>,
    pub skip_mouseup: ReadSignal<usize>,
    pub set_skip_mouseup: WriteSignal<usize>,
    pub err_signal: WriteSignal<Option<String>>,
    pub game: Box<MinesweeperClient>,
    pub ws: Option<WebSocket>,
}

impl FrontendGame {
    pub fn try_reveal(&self, row: usize, col: usize) -> Result<()> {
        let Some(player) =  self.player.get() else {
            bail!("Tried to play when not a player")
        };
        let play_json = serde_json::to_string(&Play {
            player,
            action: PlayAction::Reveal,
            point: BoardPoint { row, col },
        })?;
        self.send(play_json);
        Ok(())
    }

    pub fn try_flag(&self, row: usize, col: usize) -> Result<()> {
        let Some(player) =  self.player.get() else {
            bail!("Tried to play when not a player")
        };
        let play_json = serde_json::to_string(&Play {
            player,
            action: PlayAction::Flag,
            point: BoardPoint { row, col },
        })?;
        self.send(play_json);
        Ok(())
    }

    pub fn try_reveal_adjacent(&self, row: usize, col: usize) -> Result<()> {
        let Some(player) =  self.player.get() else {
            bail!("Tried to play when not a player")
        };
        let play_json = serde_json::to_string(&Play {
            player,
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
            GameMessage::PlayerUpdate(pu) => {
                self.game.players[pu.player_id] = Some(pu.clone());
                self.player_signals[pu.player_id](Some(pu));
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
            GameMessage::PlayersState(ps) => {
                ps.iter().cloned().for_each(|cp| {
                    if let Some(cp) = cp {
                        self.game.players[cp.player_id] = Some(cp.clone());
                        self.player_signals[cp.player_id](Some(cp));
                    }
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
            if web_socket.ready_state() != 1 {
                return;
            }
            let _ = web_socket.send_with_str(&s);
        }
    }

    pub fn close(&self) {
        if let Some(web_socket) = self.ws.borrow() {
            let _ = web_socket.close();
        }
    }
}
