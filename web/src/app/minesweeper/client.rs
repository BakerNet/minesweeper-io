use anyhow::{anyhow, bail, Result};
use leptos::*;
use std::{cell::RefCell, rc::Rc};

use minesweeper_lib::{
    board::BoardPoint,
    cell::{HiddenCell, PlayerCell},
    client::{ClientPlayer, MinesweeperClient},
    game::{Action as PlayAction, Play},
};

use crate::messages::GameMessage;

use super::GameInfo;

#[derive(Clone)]
pub struct PlayersContext {
    pub game_id: Rc<String>,
    pub is_owner: bool,
    pub has_owner: bool,
    pub player_id: ReadSignal<Option<usize>>,
    pub players: Vec<ReadSignal<Option<ClientPlayer>>>,
    pub players_loaded: ReadSignal<bool>,
    pub join_trigger: Trigger,
    pub started: ReadSignal<bool>,
}

impl PlayersContext {
    pub fn from(frontend_game: &FrontendGame) -> Self {
        PlayersContext {
            game_id: Rc::clone(&frontend_game.game_id),
            is_owner: frontend_game.is_owner,
            has_owner: frontend_game.has_owner,
            player_id: frontend_game.player_id,
            players: frontend_game.players.clone(),
            players_loaded: frontend_game.players_loaded,
            join_trigger: frontend_game.join_trigger,
            started: frontend_game.started,
        }
    }
}

#[derive(Clone)]
pub struct FrontendGame {
    pub game_id: Rc<String>,
    pub is_owner: bool,
    pub has_owner: bool,
    pub player_id: ReadSignal<Option<usize>>,
    pub players: Vec<ReadSignal<Option<ClientPlayer>>>,
    pub players_loaded: ReadSignal<bool>,
    pub err_signal: WriteSignal<Option<String>>,
    pub join_trigger: Trigger,
    pub started: ReadSignal<bool>,
    pub completed: ReadSignal<bool>,
    pub sync_time: ReadSignal<Option<usize>>,
    pub flag_count: ReadSignal<usize>,
    pub cells: Vec<Vec<ReadSignal<PlayerCell>>>,
    cell_signals: Vec<Vec<WriteSignal<PlayerCell>>>,
    set_player_id: WriteSignal<Option<usize>>,
    player_signals: Vec<WriteSignal<Option<ClientPlayer>>>,
    set_players_loaded: WriteSignal<bool>,
    set_started: WriteSignal<bool>,
    set_completed: WriteSignal<bool>,
    set_sync_time: WriteSignal<Option<usize>>,
    set_flag_count: WriteSignal<usize>,
    game: Rc<RefCell<MinesweeperClient>>,
    send: Rc<dyn Fn(&String)>,
}

impl FrontendGame {
    pub fn new(
        game_info: &GameInfo,
        err_signal: WriteSignal<Option<String>>,
        send: Rc<dyn Fn(&String)>,
    ) -> Self {
        let mut read_signals = Vec::with_capacity(game_info.rows * game_info.cols);
        let mut write_signals = Vec::with_capacity(game_info.rows * game_info.cols);
        game_info.final_board.iter().for_each(|cells| {
            let mut read_row = Vec::new();
            let mut write_row = Vec::new();
            cells.iter().for_each(|cell| {
                let (rs, ws) = create_signal(*cell);
                read_row.push(rs);
                write_row.push(ws);
            });
            read_signals.push(read_row);
            write_signals.push(write_row);
        });
        let mut players = Vec::with_capacity(game_info.players.len());
        let mut player_signals = Vec::with_capacity(game_info.players.len());
        game_info.players.iter().for_each(|p| {
            let (rs, ws) = create_signal(p.clone());
            players.push(rs);
            player_signals.push(ws);
        });
        let (players_loaded, set_players_loaded) = create_signal(false);
        let (player_id, set_player_id) = create_signal::<Option<usize>>(None);
        let join_trigger = create_trigger();
        let (started, set_started) = create_signal::<bool>(game_info.is_started);
        let (completed, set_completed) = create_signal::<bool>(game_info.is_completed);
        let (sync_time, set_sync_time) = create_signal::<Option<usize>>(None);
        let (flag_count, set_flag_count) = create_signal::<usize>(0);
        let rows = game_info.rows;
        let cols = game_info.cols;
        FrontendGame {
            game_id: Rc::new(game_info.game_id.to_owned()),
            is_owner: game_info.is_owner,
            has_owner: game_info.has_owner,
            cells: read_signals,
            cell_signals: write_signals,
            player_id,
            set_player_id,
            players,
            player_signals,
            players_loaded,
            set_players_loaded,
            err_signal,
            join_trigger,
            started,
            set_started,
            completed,
            set_completed,
            sync_time,
            set_sync_time,
            flag_count,
            set_flag_count,
            game: Rc::new(RefCell::new(MinesweeperClient::new(rows, cols))),
            send,
        }
    }

    fn play_protections(&self) -> Result<usize> {
        if !(self.started).get() || (self.completed).get() {
            bail!("Tried to play when game not active")
        }
        let Some(player) =  self.player_id.get() else {
            bail!("Tried to play when not a player")
        };
        let Some(player_info) = self.players[player]
            .get() else {
            bail!("Tried to play when player info not available")
        };
        if player_info.dead {
            bail!("Tried to play when dead")
        }
        Ok(player)
    }

    pub fn try_reveal(&self, row: usize, col: usize) -> Result<()> {
        let player = self.play_protections()?;
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
        let player = self.play_protections()?;
        let game: &MinesweeperClient = &(*self.game).borrow();
        if let PlayerCell::Revealed(_) = game.board[BoardPoint { row, col }] {
            return Ok(());
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
        let player = self.play_protections()?;
        let game: &MinesweeperClient = &(*self.game).borrow();
        if let PlayerCell::Revealed(_) = game.board[BoardPoint { row, col }] {
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
        log::debug!("{}", msg);
        let game_message = serde_json::from_str::<GameMessage>(msg)?;
        log::debug!("{:?}", game_message);
        let game: &mut MinesweeperClient = &mut (*self.game).borrow_mut();
        match game_message {
            GameMessage::PlayerId(player_id) => {
                (self.set_player_id)(Some(player_id));
                Ok(())
            }
            GameMessage::PlayOutcome(po) => {
                let plays = game.update(po);
                plays.iter().for_each(|(point, cell)| {
                    log::debug!("Play outcome: {:?} {:?}", point, cell);
                    self.update_cell(*point, *cell);
                    if game.game_over {
                        (self.set_completed)(true);
                    }
                });
                Ok(())
            }
            GameMessage::PlayerUpdate(pu) => {
                game.add_or_update_player(pu.player_id, Some(pu.score), Some(pu.dead));
                self.player_signals[pu.player_id](Some(pu));
                Ok(())
            }
            GameMessage::Error(e) => Err(anyhow!(e)),
            GameMessage::GameState(gs) => {
                let old_board = game.player_board();
                game.set_state(gs);
                game.player_board()
                    .iter()
                    .zip(old_board.iter())
                    .enumerate()
                    .for_each(|(row, (new, old))| {
                        new.iter().enumerate().for_each(|(col, cell)| {
                            if *cell != old[col] {
                                (self.cell_signals[row][col])(*cell);
                            }
                        })
                    });
                Ok(())
            }
            GameMessage::PlayersState(ps) => {
                ps.into_iter().for_each(|cp| {
                    if let Some(cp) = cp {
                        game.add_or_update_player(cp.player_id, Some(cp.score), Some(cp.dead));
                        log::debug!("Sending player signal {:?}", cp);
                        self.player_signals[cp.player_id](Some(cp));
                    }
                });
                (self.set_players_loaded)(true);
                Ok(())
            }
            GameMessage::GameStarted => {
                (self.set_started)(true);
                Ok(())
            }
            GameMessage::SyncTimer(secs) => {
                (self.set_sync_time)(Some(secs));
                Ok(())
            }
        }
    }

    pub fn update_cell(&self, point: BoardPoint, cell: PlayerCell) {
        let curr_cell = self.cells[point.row][point.col]();
        match (curr_cell, cell) {
            (PlayerCell::Hidden(HiddenCell::Flag), PlayerCell::Hidden(HiddenCell::Empty)) => {
                self.set_flag_count.update(|nm| *nm -= 1);
                log::debug!("Removed flag")
            }
            (PlayerCell::Hidden(HiddenCell::Flag), PlayerCell::Revealed(_)) => {
                self.set_flag_count.update(|nm| *nm -= 1);
                log::debug!("Removed flag")
            }
            (PlayerCell::Hidden(HiddenCell::Empty), PlayerCell::Hidden(HiddenCell::Flag)) => {
                self.set_flag_count.update(|nm| *nm += 1);
                log::debug!("Added flag")
            }
            (PlayerCell::Hidden(HiddenCell::Empty), PlayerCell::Revealed(rc))
                if rc.contents.is_mine() =>
            {
                self.set_flag_count.update(|nm| *nm += 1);
                log::debug!("Mine revealed")
            }
            _ => {}
        }
        self.cell_signals[point.row][point.col](cell);
    }

    pub fn send(&self, s: &String) {
        log::debug!("before send {s}");
        (self.send)(s)
    }
}
