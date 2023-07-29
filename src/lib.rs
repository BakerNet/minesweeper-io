mod board;

use core::fmt;
use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
};

use anyhow::{bail, Ok, Result};
use board::Board;
use rand::{seq::SliceRandom, thread_rng};

pub struct Minesweeper {
    num_mines: usize,
    available: HashSet<usize>,
    players: Vec<Player>,
    board: Board<(Cell, CellState)>,
}

impl Display for Minesweeper {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let rows = (0..self.rows)
            .collect::<Vec<usize>>()
            .iter()
            .map(|row| {
                let row_slice = &self.board[(row * self.cols)..(row * self.cols + self.cols)];
                let row_flat = row_slice
                    .into_iter()
                    .map(|item| {
                        if item.1.revealed {
                            let c = format!("{}", item.0.value().unwrap());
                            c
                        } else {
                            String::from("-")
                        }
                    })
                    .collect::<String>();
                row_flat
            })
            .fold(String::new(), |mut acc, s| {
                acc.push_str(&format!("{}\n", s));
                acc
            });
        let row_trim = rows.trim_end();
        write!(f, "{}", row_trim)
    }
}

impl Minesweeper {
    fn new(rows: usize, cols: usize, num_mines: usize, max_players: usize) -> Result<Self> {
        let total = rows * cols;
        if num_mines > total {
            bail!("Too many mines to create game");
        }
        let game = Minesweeper {
            num_mines,
            available: (0..total).collect(),
            players: vec![Player::default(); max_players],
            board: Board {
                rows,
                cols,
                board: vec![(Cell::default(), CellState::default()); total],
            },
        };
        Ok(game)
    }

    pub fn init_game(
        rows: usize,
        cols: usize,
        num_mines: usize,
        max_players: usize,
    ) -> Result<Minesweeper> {
        let mut game = Self::new(rows, cols, num_mines, max_players)?;
        let mut take_available: Vec<usize> = game.available.iter().copied().collect::<Vec<_>>();
        take_available.shuffle(&mut thread_rng());
        let indices_to_plant = &take_available[0..game.num_mines];
        indices_to_plant.iter().for_each(|x| {
            game.plant(*x);
        });
        Ok(game)
    }

    pub fn play(
        &mut self,
        player: usize,
        action: Action,
        cell_point: CellPoint,
    ) -> Result<PlayOutcome> {
        if self.available.is_empty() {
            bail!("Game is over")
        }
        if self.players[player].dead {
            bail!("Tried to play as dead player")
        }
        let index = cell_point.row * self.cols + cell_point.col;
        if cell_point.row > self.rows || cell_point.col > self.cols || index >= self.board.len() {
            bail!("Tried to play point outside of playzone")
        }
        match action {
            Action::Click => self.handle_click(player, index),
            Action::DoubleClick => self.handle_double_click(player, index),
            Action::Flag => self.handle_flag(player, index),
        }
    }

    fn handle_flag(&mut self, player: usize, index: usize) -> Result<PlayOutcome> {
        let (_, cell_state) = &self.board[index];
        if cell_state.revealed {
            bail!("Tried to play already revealed cell")
        }
        let cell_point = CellPoint {
            row: index & self.cols,
            col: index / self.cols,
        };
        if self.players[player].flags.contains(&cell_point) {
            let i = self.players[player]
                .flags
                .iter()
                .position(|&r| r == cell_point)
                .unwrap();
            self.players[player].flags.remove(i);
        }
        self.players[player].flags.push(cell_point);
        Ok(PlayOutcome::Success(Vec::new()))
    }

    fn handle_click(&mut self, player: usize, index: usize) -> Result<PlayOutcome> {
        let (_, cell_state) = &self.board[index];
        if cell_state.revealed {
            bail!("Tried to play already revealed cell")
        }
        if !(self.players[player].played) {
            self.players[player].played = true;
            self.unplant(index, true);
        }
        let cell_point = CellPoint {
            row: index & self.cols,
            col: index / self.cols,
        };
        let (cell, _) = &self.board[index];
        match cell {
            Cell::Bomb => {
                self.reveal(player, index);
                self.players[0].dead = true;
                Ok(PlayOutcome::Failure(RevealedCell {
                    cell_point,
                    player,
                    contents: self.board[index].0,
                }))
            }
            Cell::Empty(x) if x == &0 => {
                let revealed_points = self.reveal_neighbors(player, index)?;
                let revealed_points = revealed_points
                    .into_iter()
                    .map(|i| RevealedCell {
                        cell_point: CellPoint {
                            row: i / self.cols,
                            col: i % self.cols,
                        },
                        player,
                        contents: self.board[i].0,
                    })
                    .collect::<Vec<_>>();
                self.players[player].points += revealed_points.len();
                if self.available.is_empty() {
                    Ok(PlayOutcome::Victory(revealed_points))
                } else {
                    Ok(PlayOutcome::Success(revealed_points))
                }
            }
            Cell::Empty(_) => {
                self.reveal(player, index);
                self.players[player].points += 1;
                let revealed_point = vec![RevealedCell {
                    cell_point,
                    player,
                    contents: self.board[index].0,
                }];
                if self.available.is_empty() {
                    Ok(PlayOutcome::Victory(revealed_point))
                } else {
                    Ok(PlayOutcome::Success(revealed_point))
                }
            }
        }
    }

    fn handle_double_click(&mut self, player: usize, index: usize) -> Result<PlayOutcome> {
        todo!()
    }

    pub fn is_over(&self) -> bool {
        self.available.len() == 0 || self.players.iter().all(|x| x.dead)
    }

    pub fn player_board(&self, player: usize) -> Vec<Vec<PlayerCell>> {
        let mut return_board: Vec<Vec<PlayerCell>> =
            vec![vec![PlayerCell::Hidden; self.cols]; self.rows];
        for r in 0..self.rows {
            for c in 0..self.cols {
                let item = &self.board[r * self.cols + c];
                if item.1.revealed {
                    return_board[r][c] = PlayerCell::Revealed(RevealedCell {
                        cell_point: CellPoint { row: r, col: c },
                        player: item.1.player.unwrap(),
                        contents: item.0,
                    });
                }
            }
        }
        for f in self.players[player].flags.iter() {
            if let PlayerCell::Hidden = return_board[f.row][f.col] {
                return_board[f.row][f.col] = PlayerCell::Flag
            }
        }
        return_board
    }

    fn reveal(&mut self, player: usize, index: usize) -> bool {
        if self.board[index].1.revealed {
            false
        } else {
            self.board[index].1.revealed = true;
            self.board[index].1.player = Some(player);
            self.available.remove(&index);
            true
        }
    }

    fn reveal_neighbors(&mut self, player: usize, index: usize) -> Result<Vec<usize>> {
        self.reveal(player, index);
        let final_vec = vec![index];
        let neighbors = self.neighbors(index);
        neighbors.iter().try_fold(final_vec, |mut acc, i| {
            let item = self.board[*i].clone();
            if item.1.revealed {
                return Ok(acc);
            }
            if let Cell::Empty(x) = item.0 {
                if x == 0 {
                    let mut recur_acc = self.reveal_neighbors(player, *i)?;
                    acc.append(&mut recur_acc)
                } else if self.reveal(player, *i) {
                    acc.push(*i)
                }
            } else {
                bail!("Called reveal neighbors when there is a bomb nearby")
            }
            Ok(acc)
        })
    }

    fn plant(&mut self, index: usize) {
        self.available.remove(&index);

        self.board[index].0 = self.board[index].0.plant().unwrap();

        let neighbors = self.neighbors(index);
        neighbors
            .iter()
            .copied()
            .for_each(|i| self.board[i].0 = self.board[i].0.increment());
    }

    fn unplant(&mut self, index: usize, rem_neighbors: bool) {
        let neighbors = self.neighbors(index);

        let was_bomb = self.board[index].0.is_bomb();
        if was_bomb {
            let neighboring_bombs = neighbors
                .iter()
                .copied()
                .fold(0, |acc, i| acc + bool_to_u8(self.board[i].0.is_bomb()));

            self.board[index].0 = self.board[index].0.unplant(neighboring_bombs).unwrap();
        }

        neighbors.iter().copied().for_each(|i| {
            let new = if was_bomb {
                self.board[i].0.decrement()
            } else {
                self.board[i].0
            };
            if rem_neighbors && matches!(new, Cell::Bomb) {
                self.unplant(i, false);
            } else {
                self.board[i].0 = new;
            }
        });
    }
}

fn bool_to_u8(b: bool) -> u8 {
    match b {
        true => 1,
        false => 0,
    }
}

#[derive(Clone, Debug, Default)]
struct Player {
    played: bool,
    dead: bool,
    points: usize,
    flags: Vec<CellPoint>,
}

#[derive(Clone, Debug)]
pub enum PlayOutcome {
    Success(Vec<RevealedCell>),
    Failure(RevealedCell),
    Victory(Vec<RevealedCell>),
}

impl PlayOutcome {
    pub fn len(&self) -> usize {
        match self {
            Self::Success(v) => v.len(),
            Self::Victory(v) => v.len(),
            Self::Failure(_) => 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PlayerCell {
    Hidden,
    Flag,
    Revealed(RevealedCell),
}

#[derive(Clone, Copy, Debug)]
pub struct RevealedCell {
    pub cell_point: CellPoint,
    pub player: usize,
    pub contents: Cell,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellPoint {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Debug, Default)]
struct CellState {
    revealed: bool,
    player: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Flag,
    Click,
    DoubleClick,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Cell {
    Empty(u8),
    Bomb,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty(0)
    }
}

impl Cell {
    fn increment(self) -> Self {
        match self {
            Self::Empty(x) => Cell::Empty(x + 1),
            Self::Bomb => Cell::Bomb,
        }
    }

    fn decrement(self) -> Self {
        match self {
            Self::Empty(x) => Cell::Empty(x - 1),
            Self::Bomb => Cell::Bomb,
        }
    }

    fn plant(self) -> Result<Self> {
        match self {
            Self::Empty(_) => Ok(Cell::Bomb),
            Self::Bomb => bail!("Plant on bomb not allowed"),
        }
    }

    fn unplant(self, num: u8) -> Result<Self> {
        match self {
            Self::Empty(_) => bail!("Unplant on empty not allowed"),
            Self::Bomb => Ok(Cell::Empty(num)),
        }
    }

    pub fn is_bomb(&self) -> bool {
        matches!(self, Self::Bomb)
    }

    pub fn value(&self) -> Option<u8> {
        match self {
            Self::Empty(x) => Some(*x),
            Self::Bomb => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Action, Cell, CellPoint, Minesweeper, PlayOutcome};

    #[test]
    fn create_and_init_game() {
        let game = Minesweeper::init_game(9, 9, 10, 1).unwrap();
        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 10);
    }

    #[test]
    fn plant_works() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(0);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 1);
        assert_eq!(game.available.len(), 9 * 9 - 1);
        let first = game.board[0].0;
        assert!(matches!(first, Cell::Bomb));
        let right = game.board[1].0;
        assert!(matches!(right, Cell::Empty(1)));
        let below = game.board[9].0;
        assert!(matches!(below, Cell::Empty(1)));
        let right_below = game.board[10].0;
        assert!(matches!(right_below, Cell::Empty(1)));
        let two_right = game.board[2].0;
        assert!(matches!(two_right, Cell::Empty(0)));

        game.plant(10);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 2);
        assert_eq!(game.available.len(), 9 * 9 - 2);
        let first = game.board[0].0;
        assert!(matches!(first, Cell::Bomb));
        let right = game.board[1].0;
        assert!(matches!(right, Cell::Empty(2)));
        let below = game.board[9].0;
        assert!(matches!(below, Cell::Empty(2)));
        let right_below = game.board[10].0;
        assert!(matches!(right_below, Cell::Bomb));
        let two_right = game.board[2].0;
        assert!(matches!(two_right, Cell::Empty(1)));

        game.plant(11);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 3);
        assert_eq!(game.available.len(), 9 * 9 - 3);
        let first = game.board[0].0;
        assert!(matches!(first, Cell::Bomb));
        let right = game.board[1].0;
        assert!(matches!(right, Cell::Empty(3)));
        let below = game.board[9].0;
        assert!(matches!(below, Cell::Empty(2)));
        let right_below = game.board[10].0;
        assert!(matches!(right_below, Cell::Bomb));
        let two_right = game.board[2].0;
        assert!(matches!(two_right, Cell::Empty(2)));
    }

    #[test]
    fn unplant_bomb_works() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(0);
        game.plant(10);
        game.plant(11);

        game.unplant(0, true);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 1);
        assert_eq!(game.available.len(), 9 * 9 - 3);
        let clicked_bomb_loc = game.board[0].0;
        assert!(matches!(clicked_bomb_loc, Cell::Empty(0)));
        let second_bomb_loc = game.board[10].0;
        assert!(matches!(second_bomb_loc, Cell::Empty(1)));
    }

    #[test]
    fn unplant_cell_works() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(0);
        game.plant(10);
        game.plant(11);

        game.unplant(2, true);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 1);
        assert_eq!(game.available.len(), 9 * 9 - 3);
        let clicked_loc = game.board[2].0;
        assert!(matches!(clicked_loc, Cell::Empty(0)));
        let first_bomb_loc = game.board[0].0;
        assert!(matches!(first_bomb_loc, Cell::Bomb));
        let second_bomb_loc = game.board[10].0;
        assert!(matches!(second_bomb_loc, Cell::Empty(1)));
    }

    #[test]
    fn first_play_bomb_works() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(0);
        game.plant(10);
        game.plant(11);
        game.plant(19);

        let res = game
            .play(0, Action::Click, CellPoint { row: 0, col: 0 })
            .unwrap();
        assert_eq!(res.len(), 4);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 2);
        assert_eq!(game.available.len(), 9 * 9 - 6);
        let clicked_bomb_loc = &game.board[0];
        assert!(matches!(clicked_bomb_loc.0, Cell::Empty(0)));
        assert!(clicked_bomb_loc.1.revealed);
        assert_eq!(clicked_bomb_loc.1.player, Some(0));
        let second_bomb_loc = &game.board[10];
        assert_eq!(second_bomb_loc.0, Cell::Empty(2));
        assert!(second_bomb_loc.1.revealed);
        assert_eq!(second_bomb_loc.1.player, Some(0));
        let third_bomb_loc = &game.board[11];
        assert!(matches!(third_bomb_loc.0, Cell::Bomb));
        assert!(!third_bomb_loc.1.revealed);
        assert_eq!(third_bomb_loc.1.player, None);
    }

    #[test]
    fn first_play_cell_works() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(0);
        game.plant(10);
        game.plant(11);
        game.plant(19);

        let res = game.play(0, Action::Click, CellPoint { col: 7, row: 7 });
        assert_eq!(res.unwrap().len(), 9 * 9 - 8);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 4);
        assert_eq!(game.available.len(), 4); // not bomb and not revealed
        let clicked_loc = &game.board[9 * 9 - 1];
        assert!(matches!(clicked_loc.0, Cell::Empty(0)));
        assert!(clicked_loc.1.revealed);
        assert_eq!(clicked_loc.1.player, Some(0));
        let second_bomb_loc = &game.board[10];
        assert!(matches!(second_bomb_loc.0, Cell::Bomb));
        assert!(!second_bomb_loc.1.revealed);
        assert_eq!(second_bomb_loc.1.player, None);
        let third_bomb_loc = &game.board[11];
        assert!(matches!(third_bomb_loc.0, Cell::Bomb));
        assert!(!third_bomb_loc.1.revealed);
        assert_eq!(third_bomb_loc.1.player, None);
    }

    #[test]
    fn second_click_bomb_failure() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(0);
        game.plant(10);
        game.plant(11);
        game.plant(19);

        let _ = game
            .play(0, Action::Click, CellPoint { col: 0, row: 0 })
            .unwrap();

        let cell_point = CellPoint { row: 1, col: 2 };
        let res = game.play(0, Action::Click, cell_point.clone());
        assert!(matches!(res.unwrap(), PlayOutcome::Failure(_)));
    }

    #[test]
    fn second_click_cell_success() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(0);
        game.plant(10);
        game.plant(11);
        game.plant(19);

        let _ = game
            .play(0, Action::Click, CellPoint { col: 0, row: 0 })
            .unwrap();

        let cell_point = CellPoint { row: 0, col: 2 };
        let res = game.play(0, Action::Click, cell_point.clone()).unwrap();
        assert!(matches!(res.clone(), PlayOutcome::Success(_)));
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn points_work() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(0);
        game.plant(10);
        game.plant(11);
        game.plant(19);

        let _ = game
            .play(0, Action::Click, CellPoint { col: 0, row: 0 })
            .unwrap();

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 2);

        let cell_point = CellPoint { row: 0, col: 2 };
        let res = game.play(0, Action::Click, cell_point.clone());
        assert!(matches!(res.unwrap(), PlayOutcome::Success(_)));
    }

    #[test]
    fn dead_errors() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(0);
        game.plant(10);
        game.plant(11);
        game.plant(19);

        let _ = game
            .play(0, Action::Click, CellPoint { col: 0, row: 0 })
            .unwrap();
        let _ = game
            .play(0, Action::Click, CellPoint { row: 1, col: 2 })
            .unwrap();

        let res = game.play(0, Action::Click, CellPoint { row: 3, col: 3 });
        assert!(matches!(res, Err(..)));
    }

    #[test]
    fn revealed_errors() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(0);
        game.plant(10);
        game.plant(11);
        game.plant(19);

        let _ = game
            .play(0, Action::Click, CellPoint { col: 0, row: 0 })
            .unwrap();

        let res = game.play(0, Action::Click, CellPoint { row: 1, col: 1 });
        assert!(matches!(res, Err(..)));
    }

    #[test]
    fn oob_errors() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        let res = game.play(0, Action::Click, CellPoint { col: 10, row: 0 });
        assert!(matches!(res, Err(..)));

        let res = game.play(0, Action::Click, CellPoint { col: 0, row: 10 });
        assert!(matches!(res, Err(..)));
    }

    #[test]
    fn victory_works() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(0);
        game.plant(10);
        game.plant(11);
        game.plant(19);

        let _ = game
            .play(0, Action::Click, CellPoint { col: 0, row: 0 })
            .unwrap();
        let _ = game
            .play(0, Action::Click, CellPoint { row: 8, col: 8 })
            .unwrap();

        let _ = game
            .play(0, Action::Click, CellPoint { row: 0, col: 2 })
            .unwrap();
        let res = game
            .play(0, Action::Click, CellPoint { row: 2, col: 0 })
            .unwrap();
        assert!(matches!(res, PlayOutcome::Victory(..)));
        assert_eq!(game.players[0].points, 79);
    }
}
