use crate::board::{Board, BoardPoint};
use crate::cell::PlayerCell;
use crate::game::{Action, PlayOutcome};

use serde::{Deserialize, Serialize};

pub struct MinesweeperClient {
    pub player: Option<(String, usize)>,
    pub dead: bool,
    pub game_over: bool,
    pub board: Board<PlayerCell>,
}

impl MinesweeperClient {
    pub fn new(rows: usize, cols: usize) -> Self {
        let board = Board::new(rows, cols, PlayerCell::default());
        let game = MinesweeperClient {
            player: None,
            dead: false,
            game_over: false,
            board,
        };
        game
    }

    pub fn set_state(&mut self, from_vec: Vec<Vec<PlayerCell>>) {
        self.board = Board::from_vec(from_vec)
    }

    pub fn player_board(&self) -> Vec<Vec<PlayerCell>> {
        (&self.board).into()
    }

    pub fn update(&mut self, play_outcome: PlayOutcome) -> Vec<(BoardPoint, PlayerCell)> {
        let mut updated = Vec::new();
        match play_outcome {
            PlayOutcome::Success(cells) => cells.into_iter().for_each(|cell| {
                let point = cell.cell_point;
                let player_cell = PlayerCell::Revealed(cell);
                self.board[point] = player_cell;
                updated.push((point, player_cell));
            }),
            PlayOutcome::Victory(cells) => {
                cells.into_iter().for_each(|cell| {
                    let point = cell.cell_point;
                    let player_cell = PlayerCell::Revealed(cell);
                    self.board[point] = player_cell;
                    updated.push((point, player_cell));
                });
                self.game_over = true;
            }
            PlayOutcome::Failure(cell) => {
                self.dead = self.is_player(cell.player);
                let point = cell.cell_point;
                let player_cell = PlayerCell::Revealed(cell);
                self.board[point] = player_cell;
                updated.push((point, player_cell));
            }
            PlayOutcome::Flag(item) => {
                let point = item.0;
                let player_cell = item.1;
                self.board[point] = player_cell;
                updated.push(item);
            }
        }
        updated
    }

    fn is_player(&self, player: usize) -> bool {
        match &self.player {
            None => false,
            Some(p) => p.1 == player,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Play {
    pub player: usize,
    pub action: Action,
    pub point: BoardPoint,
}

#[cfg(test)]
mod test {}
