use anyhow::Result;
use leptos::prelude::*;
use std::sync::{Arc, RwLock};

use minesweeper_lib::{
    board::{Board, BoardPoint},
    cell::{HiddenCell, PlayerCell},
    client::MinesweeperClient,
    game::{Action as PlayAction, Minesweeper, MinesweeperBuilder, MinesweeperOpts, Play},
};

use game_ui::GameInfo;

#[derive(Clone)]
pub struct FrontendGame {
    pub err_signal: WriteSignal<Option<String>>,
    pub completed: ReadSignal<bool>,
    pub victory: ReadSignal<bool>,
    pub dead: ReadSignal<bool>,
    pub flag_count: ReadSignal<usize>,
    pub sync_time: ReadSignal<Option<usize>>,
    pub cells: Arc<Vec<Vec<ReadSignal<PlayerCell>>>>,
    cell_signals: Arc<Vec<Vec<WriteSignal<PlayerCell>>>>,
    set_completed: WriteSignal<bool>,
    set_victory: WriteSignal<bool>,
    set_dead: WriteSignal<bool>,
    set_flag_count: WriteSignal<usize>,
    set_sync_time: WriteSignal<Option<usize>>,
    game: Arc<RwLock<Minesweeper>>,
    game_client: Arc<RwLock<MinesweeperClient>>,
}

impl FrontendGame {
    pub fn new(game_info: &GameInfo, err_signal: WriteSignal<Option<String>>) -> Self {
        let (read_signals, write_signals) = signals_from_board(&game_info.board());
        let (completed, set_completed) = signal(game_info.is_completed);
        let (victory, set_victory) = signal(false);
        let (dead, set_dead) = signal(false);
        let (flag_count, set_flag_count) = signal(0);
        let (sync_time, set_sync_time) = signal::<Option<usize>>(None);
        let rows = game_info.rows;
        let cols = game_info.cols;
        let num_mines = game_info.num_mines;
        FrontendGame {
            cells: read_signals.into(),
            cell_signals: write_signals.into(),
            err_signal,
            completed,
            victory,
            dead,
            set_completed,
            set_victory,
            set_dead,
            flag_count,
            set_flag_count,
            sync_time,
            set_sync_time,
            game: Arc::new(RwLock::new(
                MinesweeperBuilder::new(MinesweeperOpts {
                    rows,
                    cols,
                    num_mines,
                })
                .expect("Should be able to create game")
                .with_log()
                .with_superclick()
                .init(),
            )),
            game_client: Arc::new(RwLock::new(MinesweeperClient::new(rows, cols))),
        }
    }

    pub fn try_reveal(&self, row: usize, col: usize) -> Result<()> {
        let point = BoardPoint { row, col };
        let play = Play {
            player: 0,
            action: PlayAction::Reveal,
            point,
        };
        self.try_play(play)
    }

    pub fn try_flag(&self, row: usize, col: usize) -> Result<()> {
        let point = BoardPoint { row, col };
        let play = Play {
            player: 0,
            action: PlayAction::Flag,
            point,
        };
        self.try_play(play)
    }

    pub fn try_reveal_adjacent(&self, row: usize, col: usize) -> Result<()> {
        let point = BoardPoint { row, col };
        let play = Play {
            player: 0,
            action: PlayAction::RevealAdjacent,
            point,
        };
        self.try_play(play)
    }

    fn try_play(&self, play: Play) -> Result<()> {
        let is_started = self.sync_time.get_untracked().is_some();
        let game_client: &mut MinesweeperClient = &mut (*self.game_client).write().unwrap();
        let game: &mut Minesweeper = &mut (*self.game).write().unwrap();
        let po = game.play(play)?;
        let plays = game_client.update(po);
        game_client.add_or_update_player(0, game.player_score(0).ok(), game.player_dead(0).ok());
        if !is_started {
            self.set_sync_time.set(Some(0));
        }
        plays.iter().for_each(|(point, cell)| {
            log::debug!("Play outcome: {point:?} {cell:?}");
            self.update_cell(*point, *cell);
        });
        let is_victory = game_client.victory;
        let is_dead = matches!(game.player_dead(0), Ok(true));
        
        if is_victory {
            self.set_victory.set(true);
            self.set_completed.set(true);
        } else if is_dead {
            self.set_dead.set(true);
            self.set_completed.set(true);
        }
        Ok(())
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
