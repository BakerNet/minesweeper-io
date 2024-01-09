use std::{borrow::Borrow, cell::RefCell, rc::Rc};

use anyhow::{anyhow, bail, Result};
use leptos::*;
use minesweeper_lib::{
    board::BoardPoint,
    cell::PlayerCell,
    client::{ClientPlayer, MinesweeperClient, Play},
    game::{Action as PlayAction, PlayOutcome},
};
use serde::{Deserialize, Serialize};

use super::GameInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "game_message", content = "data")]
pub enum GameMessage {
    PlayerId(usize),
    PlayOutcomes(PlayOutcome),
    PlayerUpdate(ClientPlayer),
    GameState(Vec<Vec<PlayerCell>>),
    PlayersState(Vec<Option<ClientPlayer>>),
    Error(String),
}

impl GameMessage {
    pub fn to_string(self) -> String {
        serde_json::to_string::<GameMessage>(&self)
            .unwrap_or_else(|_| panic!("Should be able to serialize GameMessage {:?}", self))
    }
}

#[derive(Clone)]
pub struct FrontendGame {
    pub game_info: GameInfo,
    pub cell_signals: Vec<Vec<WriteSignal<PlayerCell>>>,
    pub player_id: ReadSignal<Option<usize>>,
    pub set_player_id: WriteSignal<Option<usize>>,
    pub players: Vec<ReadSignal<Option<ClientPlayer>>>,
    pub player_signals: Vec<WriteSignal<Option<ClientPlayer>>>,
    pub skip_mouseup: ReadSignal<usize>,
    pub set_skip_mouseup: WriteSignal<usize>,
    pub err_signal: WriteSignal<Option<String>>,
    pub game: Rc<RefCell<MinesweeperClient>>,
    send: Rc<dyn Fn(&str)>,
    close: Rc<dyn Fn()>,
}

impl FrontendGame {
    pub fn new(
        game_info: GameInfo,
        err_signal: WriteSignal<Option<String>>,
        send: Rc<dyn Fn(&str)>,
        close: Rc<dyn Fn()>,
    ) -> (Self, Vec<Vec<ReadSignal<PlayerCell>>>) {
        let mut read_signals: Vec<Vec<ReadSignal<PlayerCell>>> = Vec::new();
        let mut write_signals: Vec<Vec<WriteSignal<PlayerCell>>> = Vec::new();
        (0..game_info.rows).for_each(|_| {
            let mut read_row = Vec::new();
            let mut write_row = Vec::new();
            (0..game_info.cols).for_each(|_| {
                let (rs, ws) = create_signal(PlayerCell::Hidden);
                read_row.push(rs);
                write_row.push(ws);
            });
            read_signals.push(read_row);
            write_signals.push(write_row);
        });
        let mut players: Vec<ReadSignal<Option<ClientPlayer>>> = Vec::new();
        let mut player_signals: Vec<WriteSignal<Option<ClientPlayer>>> = Vec::new();
        (0..game_info.max_players).for_each(|_| {
            let (rs, ws) = create_signal(None);
            players.push(rs);
            player_signals.push(ws);
        });
        let (player_id, set_player_id) = create_signal::<Option<usize>>(None);
        let (skip_mouseup, set_skip_mouseup) = create_signal::<usize>(0);
        let rows = game_info.rows;
        let cols = game_info.cols;
        (
            FrontendGame {
                game_info,
                cell_signals: write_signals,
                player_id,
                set_player_id,
                players,
                player_signals,
                skip_mouseup,
                set_skip_mouseup,
                err_signal,
                game: Rc::new(RefCell::new(MinesweeperClient::new(rows, cols))),
                send,
                close,
            },
            read_signals,
        )
    }

    pub fn try_reveal(&self, row: usize, col: usize) -> Result<()> {
        let Some(player) =  self.player_id.get() else {
            bail!("Tried to play when not a player")
        };
        let game: &MinesweeperClient = &(*self.game).borrow();
        if let PlayerCell::Revealed(_) = game.board[BoardPoint { row, col }] {
            bail!("Tried to click revealed cell")
        }
        let play_json = serde_json::to_string(&Play {
            player,
            action: PlayAction::Reveal,
            point: BoardPoint { row, col },
        })?;
        self.send(&play_json);
        Ok(())
    }

    pub fn try_flag(&self, row: usize, col: usize) -> Result<()> {
        let Some(player) =  self.player_id.get() else {
            bail!("Tried to play when not a player")
        };
        let game: &MinesweeperClient = &(*self.game).borrow();
        if let PlayerCell::Revealed(_) = game.borrow().board[BoardPoint { row, col }] {
            bail!("Tried to flag revealed cell")
        }
        let play_json = serde_json::to_string(&Play {
            player,
            action: PlayAction::Flag,
            point: BoardPoint { row, col },
        })?;
        self.send(&play_json);
        Ok(())
    }

    pub fn try_reveal_adjacent(&self, row: usize, col: usize) -> Result<()> {
        let Some(player) =  self.player_id.get() else {
            bail!("Tried to play when not a player")
        };
        let game: &MinesweeperClient = &(*self.game).borrow();
        if let PlayerCell::Revealed(_) = game.borrow().board[BoardPoint { row, col }] {
        } else {
            bail!("Tried to reveal adjacent for hidden cell")
        }
        if !game.neighbors_flagged(BoardPoint { row, col }) {
            bail!("Tried to reveal adjacent with wrong number of flags")
        }
        let play_json = serde_json::to_string(&Play {
            player,
            action: PlayAction::RevealAdjacent,
            point: BoardPoint { row, col },
        })?;
        self.send(&play_json);
        Ok(())
    }

    pub fn handle_message(&self, msg: &str) -> Result<()> {
        leptos_dom::log!("{}", msg);
        let game_message = serde_json::from_str::<GameMessage>(msg)?;
        leptos_dom::log!("{:?}", game_message);
        let game: &mut MinesweeperClient = &mut (*self.game).borrow_mut();
        match game_message {
            GameMessage::PlayerId(player_id) => {
                (self.set_player_id)(Some(player_id));
                Ok(())
            }
            GameMessage::PlayOutcomes(po) => {
                let plays = game.update(po);
                plays.iter().for_each(|(point, cell)| {
                    leptos_dom::log!("{:?} {:?}", point, cell);
                    self.update_cell(*point, *cell);
                    if game.game_over {
                        self.close();
                    }
                });
                Ok(())
            }
            GameMessage::PlayerUpdate(pu) => {
                game.players[pu.player_id] = Some(pu.clone());
                self.player_signals[pu.player_id](Some(pu));
                Ok(())
            }
            GameMessage::Error(e) => Err(anyhow!(e)),
            GameMessage::GameState(gs) => {
                game.set_state(gs);
                game.player_board()
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
                ps.into_iter().for_each(|cp| {
                    if let Some(cp) = cp {
                        game.players[cp.player_id] = Some(cp.clone());
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

    pub fn send(&self, s: &str) {
        log::debug!("before send");
        (self.send)(s)
    }

    pub fn close(&self) {
        log::debug!("before close");
        (self.close)()
    }
}
