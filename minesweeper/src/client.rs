use crate::board::{Board, BoardPoint};
use crate::cell::PlayerCell;
use crate::game::{Action, PlayOutcome};

use serde::{Deserialize, Serialize};

pub struct MinesweeperClient {
    pub player: Option<usize>,
    pub players: Vec<Option<ClientPlayer>>,
    pub game_over: bool,
    pub board: Board<PlayerCell>,
}

impl MinesweeperClient {
    pub fn new(rows: usize, cols: usize) -> Self {
        let board = Board::new(rows, cols, PlayerCell::default());
        let players = vec![None; 8];
        MinesweeperClient {
            player: None,
            players,
            game_over: false,
            board,
        }
    }

    pub fn set_state(&mut self, from_vec: Vec<Vec<PlayerCell>>) {
        self.board = Board::from_vec(from_vec)
    }

    pub fn player_board(&self) -> Vec<Vec<PlayerCell>> {
        (&self.board).into()
    }

    pub fn join(&mut self, player_id: usize) {
        self.player = Some(player_id)
    }

    pub fn add_or_update_player(
        &mut self,
        player: usize,
        score: Option<usize>,
        dead: Option<bool>,
    ) {
        if let Some(p) = &mut self.players[player] {
            if let Some(score) = score {
                p.score = score;
            }
            if let Some(dead) = dead {
                p.dead = dead;
            }
        } else {
            let mut client_player = ClientPlayer::default();
            if let Some(score) = score {
                client_player.score = score;
            }
            if let Some(dead) = dead {
                client_player.dead = dead;
            }
            self.players[player] = Some(client_player)
        }
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
            Some(p) => *p == player,
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Play {
    pub player: usize,
    pub action: Action,
    pub point: BoardPoint,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ClientPlayer {
    pub player_id: usize,
    pub username: String,
    pub dead: bool,
    pub score: usize,
}

#[cfg(test)]
mod test {}
