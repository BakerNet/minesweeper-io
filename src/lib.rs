use std::collections::HashSet;

use anyhow::{bail, Result};
use rand::{seq::SliceRandom, thread_rng};

pub struct Minesweeper {
    rows: usize,
    cols: usize,
    num_mines: usize,
    available: HashSet<usize>,
    players: Vec<Player>,
    board: Vec<(Cell, CellState)>,
}

impl Minesweeper {
    fn new(rows: usize, cols: usize, num_mines: usize, max_players: usize) -> Result<Self> {
        let total = rows * cols;
        if num_mines > total {
            bail!("Too many mines to create game");
        }
        let game = Minesweeper {
            rows,
            cols,
            num_mines,
            available: (0..total).collect(),
            players: vec![Player::default(); max_players],
            board: vec![(Cell::default(), CellState::default()); total],
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

    pub fn play(&mut self, player: usize, cell_point: CellPoint) -> Result<PlayOutcome> {
        if self.players[player].dead {
            bail!("Tried to play as dead player")
        }
        let index = cell_point.row * self.cols + cell_point.col;
        if index >= self.board.len() || cell_point.row > self.rows || cell_point.col > self.cols {
            bail!("Tried to play point outside of playzone")
        }
        let (_, cell_state) = &self.board[index];
        if cell_state.revealed {
            bail!("Tried to play already revealed cell")
        }
        if !(self.players[player].played) {
            self.players[player].played = true;
            self.unplant(index, true);
        }
        let (cell, _) = &self.board[index];
        match cell {
            Cell::Bomb => {
                self.reveal(player, index);
                self.players[0].dead = true;
                Ok(PlayOutcome::Failure(cell_point))
            }
            Cell::Empty(x) if x == &0 => {
                let revealed_points = self.reveal_neighbors(player, index)?;
                let revealed_points = revealed_points
                    .into_iter()
                    .map(|i| CellPoint {
                        col: i % self.cols,
                        row: i / self.cols,
                    })
                    .collect::<Vec<_>>();
                self.players[player].points += revealed_points.len();
                Ok(PlayOutcome::Success(revealed_points))
            }
            Cell::Empty(_) => {
                self.reveal(player, index);
                self.players[player].points += 1;
                Ok(PlayOutcome::Success(vec![cell_point]))
            }
        }
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
            let revealed = self.reveal(player, *i);
            if let Cell::Empty(x) = item.0 {
                if x == 0 {
                    let mut recur_acc = self.reveal_neighbors(player, *i)?;
                    acc.append(&mut recur_acc)
                } else if revealed {
                    acc.push(*i)
                }
            } else {
                bail!("Called reveal neighbors when there is a bomb nearby")
            }
            Ok(acc)
        })
    }

    fn neighbors(&self, index: usize) -> Vec<usize> {
        let mut neighbors = Vec::<usize>::new();

        let col = index % self.cols;
        let row = index / self.cols;
        if col > 0 {
            neighbors.push(index - 1);
            if row > 0 {
                neighbors.push(index - 1 - self.cols);
            }
            if row < self.cols - 1 {
                neighbors.push(index - 1 + self.cols);
            }
        }
        if col < self.cols - 1 {
            neighbors.push(index + 1);
            if row > 0 {
                neighbors.push(index + 1 - self.cols);
            }
            if row < self.cols - 1 {
                neighbors.push(index + 1 + self.cols);
            }
        }
        if row > 0 {
            neighbors.push(index - self.cols);
        }
        if row < self.rows - 1 {
            neighbors.push(index + self.cols);
        }
        neighbors
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

#[derive(Clone, Debug)]
struct Player {
    played: bool,
    dead: bool,
    points: usize,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            played: false,
            dead: false,
            points: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub enum PlayOutcome {
    Success(Vec<CellPoint>),
    Failure(CellPoint),
}

impl PlayOutcome {
    pub fn len(&self) -> usize {
        match self {
            PlayOutcome::Success(v) => v.len(),
            PlayOutcome::Failure(_) => 1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CellPoint {
    row: usize,
    col: usize,
}

#[derive(Clone, Debug)]
struct CellState {
    revealed: bool,
    player: Option<usize>,
}

impl Default for CellState {
    fn default() -> Self {
        CellState {
            revealed: false,
            player: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cell {
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
            Cell::Empty(x) => Cell::Empty(x + 1),
            Cell::Bomb => Cell::Bomb,
        }
    }

    fn decrement(self) -> Self {
        match self {
            Cell::Empty(x) => Cell::Empty(x - 1),
            Cell::Bomb => Cell::Bomb,
        }
    }

    fn plant(self) -> Result<Self> {
        match self {
            Cell::Empty(_) => Ok(Cell::Bomb),
            Cell::Bomb => bail!("Plant on bomb not allowed"),
        }
    }

    fn unplant(self, num: u8) -> Result<Self> {
        match self {
            Cell::Empty(_) => bail!("Unplant on empty not allowed"),
            Cell::Bomb => Ok(Cell::Empty(num)),
        }
    }

    fn is_bomb(&self) -> bool {
        matches!(self, Cell::Bomb)
    }
}

#[cfg(test)]
mod test {
    use crate::{Cell, CellPoint, Minesweeper, PlayOutcome};

    #[test]
    fn create_and_init_game() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();
        game.init_game();
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

        let res = game.play(0, CellPoint { col: 0, row: 0 });
        assert_eq!(res.unwrap().len(), 4);

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

        let res = game.play(0, CellPoint { col: 7, row: 7 });
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

        let _ = game.play(0, CellPoint { col: 0, row: 0 });

        let cell_point = CellPoint { row: 1, col: 2 };
        let res = game.play(0, cell_point.clone());
        assert!(matches!(res.unwrap(), PlayOutcome::Failure(_)));
    }

    #[test]
    fn second_click_cell_success() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(0);
        game.plant(10);
        game.plant(11);
        game.plant(19);

        let _ = game.play(0, CellPoint { col: 0, row: 0 });

        let cell_point = CellPoint { row: 0, col: 2 };
        let res = game.play(0, cell_point.clone());
        let res = res.unwrap();
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

        let _ = game.play(0, CellPoint { col: 0, row: 0 });

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 2);

        let cell_point = CellPoint { row: 0, col: 2 };
        let res = game.play(0, cell_point.clone());
        assert!(matches!(res.unwrap(), PlayOutcome::Success(_)));
    }
}
