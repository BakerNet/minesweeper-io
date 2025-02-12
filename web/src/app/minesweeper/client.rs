use anyhow::{anyhow, bail, Result};
use leptos::prelude::*;
use std::sync::{Arc, RwLock};

use minesweeper_lib::{
    board::{Board, BoardPoint},
    cell::{HiddenCell, PlayerCell},
    client::{ClientPlayer, MinesweeperClient},
    game::{Action as PlayAction, Play},
};

use game_manager::{ClientMessage, GameMessage};

use game_ui::GameInfo;

#[derive(Clone)]
pub struct FrontendGame {
    pub game_id: Arc<String>,
    pub is_owner: bool,
    pub has_owner: bool,
    pub player_id: ReadSignal<Option<usize>>,
    pub players: Arc<Vec<ReadSignal<Option<ClientPlayer>>>>,
    pub players_loaded: ReadSignal<bool>,
    pub err_signal: WriteSignal<Option<String>>,
    pub join_trigger: Trigger,
    pub started: ReadSignal<bool>,
    pub completed: ReadSignal<bool>,
    pub top_score: ReadSignal<Option<usize>>,
    pub sync_time: ReadSignal<Option<usize>>,
    pub flag_count: ReadSignal<usize>,
    pub cells: Arc<Vec<Vec<ReadSignal<PlayerCell>>>>,
    cell_signals: Arc<Vec<Vec<WriteSignal<PlayerCell>>>>,
    set_player_id: WriteSignal<Option<usize>>,
    player_signals: Arc<Vec<WriteSignal<Option<ClientPlayer>>>>,
    set_players_loaded: WriteSignal<bool>,
    set_started: WriteSignal<bool>,
    set_completed: WriteSignal<bool>,
    set_top_score: WriteSignal<Option<usize>>,
    set_sync_time: WriteSignal<Option<usize>>,
    set_flag_count: WriteSignal<usize>,
    game: Arc<RwLock<MinesweeperClient>>,
    send: Arc<dyn Fn(&ClientMessage) + Send + Sync>,
}

impl FrontendGame {
    pub fn new(
        game_info: &GameInfo,
        err_signal: WriteSignal<Option<String>>,
        send: Arc<dyn Fn(&ClientMessage) + Send + Sync>,
    ) -> Self {
        let (read_signals, write_signals) = signals_from_board(&game_info.final_board);
        let mut players = Vec::with_capacity(game_info.players.len());
        let mut player_signals = Vec::with_capacity(game_info.players.len());
        game_info.players.iter().for_each(|p| {
            let (rs, ws) = signal(p.clone());
            players.push(rs);
            player_signals.push(ws);
        });
        let (players_loaded, set_players_loaded) = signal(false);
        let (player_id, set_player_id) = signal::<Option<usize>>(None);
        let join_trigger = Trigger::new();
        let (started, set_started) = signal(game_info.is_started);
        let (completed, set_completed) = signal(game_info.is_completed);
        let (top_score, set_top_score) = signal::<Option<usize>>(None);
        let (sync_time, set_sync_time) = signal::<Option<usize>>(None);
        let (flag_count, set_flag_count) = signal(0);
        let rows = game_info.rows;
        let cols = game_info.cols;
        FrontendGame {
            game_id: Arc::new(game_info.game_id.to_owned()),
            is_owner: game_info.is_owner,
            has_owner: game_info.has_owner,
            cells: read_signals.into(),
            cell_signals: write_signals.into(),
            player_id,
            set_player_id,
            players: players.into(),
            player_signals: player_signals.into(),
            players_loaded,
            set_players_loaded,
            err_signal,
            join_trigger,
            started,
            set_started,
            completed,
            set_completed,
            top_score,
            set_top_score,
            sync_time,
            set_sync_time,
            flag_count,
            set_flag_count,
            game: Arc::new(RwLock::new(MinesweeperClient::new(rows, cols))),
            send,
        }
    }

    fn play_protections(&self) -> Result<usize> {
        if !(self.started).get_untracked() || (self.completed).get_untracked() {
            bail!("Tried to play when game not active")
        }
        let Some(player) = self.player_id.get_untracked() else {
            bail!("Tried to play when not a player")
        };
        let Some(player_info) = self.players[player].get_untracked() else {
            bail!("Tried to play when player info not available")
        };
        if player_info.dead {
            bail!("Tried to play when dead")
        }
        Ok(player)
    }

    pub fn try_reveal(&self, row: usize, col: usize) -> Result<()> {
        let player = self.play_protections()?;
        let game: &MinesweeperClient = &(*self.game).read().unwrap();
        let point = BoardPoint { row, col };
        if let PlayerCell::Revealed(_) = game.board[&point] {
            bail!("Tried to click revealed cell")
        }
        let play_message = ClientMessage::Play(Play {
            player,
            action: PlayAction::Reveal,
            point,
        });
        self.send(play_message);
        Ok(())
    }

    pub fn try_flag(&self, row: usize, col: usize) -> Result<()> {
        let player = self.play_protections()?;
        let game: &MinesweeperClient = &(*self.game).read().unwrap();
        let point = BoardPoint { row, col };
        if let PlayerCell::Revealed(_) = game.board[&point] {
            return Ok(());
        }
        let play_message = ClientMessage::Play(Play {
            player,
            action: PlayAction::Flag,
            point,
        });
        self.send(play_message);
        Ok(())
    }

    pub fn try_reveal_adjacent(&self, row: usize, col: usize) -> Result<()> {
        let player = self.play_protections()?;
        let game: &MinesweeperClient = &(*self.game).read().unwrap();
        let point = BoardPoint { row, col };
        if let PlayerCell::Revealed(_) = game.board[&point] {
        } else {
            bail!("Tried to reveal adjacent for hidden cell")
        }
        if !game.neighbors_flagged(&point) {
            bail!("Tried to reveal adjacent with wrong number of flags")
        }
        let play_message = ClientMessage::Play(Play {
            player,
            action: PlayAction::RevealAdjacent,
            point,
        });
        self.send(play_message);
        Ok(())
    }

    pub fn handle_message(&self, game_message: GameMessage) -> Result<()> {
        let game: &mut MinesweeperClient = &mut (*self.game).write().unwrap();
        match game_message {
            GameMessage::PlayerId(player_id) => {
                self.set_player_id.set(Some(player_id));
                Ok(())
            }
            GameMessage::PlayOutcome(po) => {
                let plays = game.update(po);
                plays.iter().for_each(|(point, cell)| {
                    log::debug!("Play outcome: {:?} {:?}", point, cell);
                    self.update_cell(*point, *cell);
                });
                if game.game_over {
                    self.set_completed.set(true);
                }
                Ok(())
            }
            GameMessage::PlayerUpdate(pu) => {
                game.add_or_update_player(pu.player_id, Some(pu.score), Some(pu.dead));
                self.player_signals[pu.player_id].set(Some(pu));
                Ok(())
            }
            GameMessage::TopScore(s) => {
                self.set_top_score.set(Some(s));
                Ok(())
            }
            GameMessage::Error(e) => Err(anyhow!(e)),
            GameMessage::GameState(gs) => {
                let old_board = game.player_board().to_owned();
                let mut flag_count = self.flag_count.get_untracked();
                game.set_state(gs);
                game.player_board()
                    .rows_iter()
                    .zip(old_board.rows_iter())
                    .enumerate()
                    .for_each(|(row, (new, old))| {
                        new.iter().enumerate().for_each(|(col, cell)| {
                            if *cell != old[col] {
                                self.cell_signals[row][col].set(*cell);
                            }
                            if matches!(cell, PlayerCell::Hidden(HiddenCell::Flag)) {
                                flag_count += 1;
                            }
                        })
                    });
                if flag_count > 0 {
                    self.set_flag_count.set(flag_count);
                }
                Ok(())
            }
            GameMessage::PlayersState(ps) => {
                ps.into_iter().for_each(|cp| {
                    if let Some(cp) = cp {
                        game.add_or_update_player(cp.player_id, Some(cp.score), Some(cp.dead));
                        log::debug!("Sending player signal {:?}", cp);
                        self.player_signals[cp.player_id].set(Some(cp));
                    }
                });
                self.set_players_loaded.set(true);
                Ok(())
            }
            GameMessage::GameStarted => {
                self.set_started.set(true);
                Ok(())
            }
            GameMessage::SyncTimer(secs) => {
                self.set_sync_time.set(Some(secs));
                Ok(())
            }
        }
    }

    pub fn update_cell(&self, point: BoardPoint, cell: PlayerCell) {
        let curr_cell = self.cells[point.row][point.col].get_untracked();
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
        self.cell_signals[point.row][point.col].set(cell);
    }

    pub fn send(&self, m: ClientMessage) {
        log::debug!("before send {m:?}");
        (self.send)(&m)
    }
}

#[allow(clippy::type_complexity)]
pub fn signals_from_board(
    board: &Board<PlayerCell>,
) -> (
    Vec<Vec<ReadSignal<PlayerCell>>>,
    Vec<Vec<WriteSignal<PlayerCell>>>,
) {
    let mut read_signals = Vec::with_capacity(board.size());
    let mut write_signals = Vec::with_capacity(board.size());
    board.rows_iter().for_each(|cells| {
        let mut read_row = Vec::new();
        let mut write_row = Vec::new();
        cells.iter().for_each(|cell| {
            let (rs, ws) = signal(*cell);
            read_row.push(rs);
            write_row.push(ws);
        });
        read_signals.push(read_row);
        write_signals.push(write_row);
    });
    (read_signals, write_signals)
}
