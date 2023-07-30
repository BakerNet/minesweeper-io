use std::collections::HashSet;

use crate::board::{Board, BoardPoint};
use crate::cell::{Cell, CellState, PlayerCell, RevealedCell};

use anyhow::{bail, Ok, Result};
use rand::{seq::SliceRandom, thread_rng};

pub struct Minesweeper {
    num_mines: usize,
    available: HashSet<BoardPoint>,
    players: Vec<Player>,
    board: Board<(Cell, CellState)>,
}

impl Minesweeper {
    fn new(rows: usize, cols: usize, num_mines: usize, max_players: usize) -> Result<Self> {
        let total = rows * cols;
        if num_mines > total {
            bail!("Too many mines to create game");
        }
        let board = Board::new(rows, cols, (Cell::default(), CellState::default()));
        let game = Minesweeper {
            num_mines,
            available: (0..total).map(|x| board.point_from_index(x)).collect(),
            players: vec![Player::default(); max_players],
            board,
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
        let mut take_available: Vec<BoardPoint> =
            game.available.iter().copied().collect::<Vec<_>>();
        take_available.shuffle(&mut thread_rng());
        let points_to_plant = &take_available[0..game.num_mines];
        points_to_plant.iter().for_each(|x| {
            game.plant(*x);
        });
        Ok(game)
    }

    pub fn play(
        &mut self,
        player: usize,
        action: Action,
        cell_point: BoardPoint,
    ) -> Result<PlayOutcome> {
        if self.available.is_empty() {
            bail!("Game is over")
        }
        if self.players[player].dead {
            bail!("Tried to play as dead player")
        }
        if !self.board.is_in_bounds(cell_point) {
            bail!("Tried to play point outside of playzone")
        }
        match action {
            Action::Click => self.handle_click(player, cell_point),
            Action::DoubleClick => self.handle_double_click(player, cell_point),
            Action::Flag => self.handle_flag(player, cell_point),
        }
    }

    fn handle_flag(&mut self, player: usize, cell_point: BoardPoint) -> Result<PlayOutcome> {
        let (_, cell_state) = &self.board[cell_point];
        if cell_state.revealed {
            bail!("Tried to play already revealed cell")
        }
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

    fn handle_click(&mut self, player: usize, cell_point: BoardPoint) -> Result<PlayOutcome> {
        let (_, cell_state) = &self.board[cell_point];
        if cell_state.revealed {
            bail!("Tried to play already revealed cell")
        }
        if !(self.players[player].played) {
            self.players[player].played = true;
            self.unplant(cell_point, true);
        }
        let (cell, _) = &self.board[cell_point];
        match cell {
            Cell::Bomb => {
                self.reveal(player, cell_point);
                self.players[0].dead = true;
                Ok(PlayOutcome::Failure(RevealedCell {
                    cell_point,
                    player,
                    contents: self.board[cell_point].0,
                }))
            }
            Cell::Empty(x) if x == &0 => {
                let revealed_points = self.reveal_neighbors(player, cell_point)?;
                let revealed_points = revealed_points
                    .into_iter()
                    .map(|c| RevealedCell {
                        cell_point: c,
                        player,
                        contents: self.board[c].0,
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
                self.reveal(player, cell_point);
                self.players[player].points += 1;
                let revealed_point = vec![RevealedCell {
                    cell_point,
                    player,
                    contents: self.board[cell_point].0,
                }];
                if self.available.is_empty() {
                    Ok(PlayOutcome::Victory(revealed_point))
                } else {
                    Ok(PlayOutcome::Success(revealed_point))
                }
            }
        }
    }

    fn handle_double_click(
        &mut self,
        player: usize,
        cell_point: BoardPoint,
    ) -> Result<PlayOutcome> {
        todo!()
    }

    pub fn is_over(&self) -> bool {
        self.available.len() == 0 || self.players.iter().all(|x| x.dead)
    }

    pub fn player_board(&self, player: usize) -> Vec<Vec<PlayerCell>> {
        let mut return_board: Vec<Vec<PlayerCell>> =
            vec![vec![PlayerCell::Hidden; self.board.cols()]; self.board.rows()];
        for r in 0..self.board.rows() {
            for c in 0..self.board.cols() {
                let point = BoardPoint { row: r, col: c };
                let item = &self.board[point];
                if item.1.revealed {
                    return_board[r][c] = PlayerCell::Revealed(RevealedCell {
                        cell_point: point,
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

    fn reveal(&mut self, player: usize, cell_point: BoardPoint) -> bool {
        if self.board[cell_point].1.revealed {
            false
        } else {
            self.board[cell_point].1.revealed = true;
            self.board[cell_point].1.player = Some(player);
            self.available.remove(&cell_point);
            true
        }
    }

    fn reveal_neighbors(
        &mut self,
        player: usize,
        cell_point: BoardPoint,
    ) -> Result<Vec<BoardPoint>> {
        self.reveal(player, cell_point);
        let final_vec = vec![cell_point];
        let neighbors = self.board.neighbors(cell_point);
        neighbors.iter().try_fold(final_vec, |mut acc, c| {
            let item = self.board[*c].clone();
            if item.1.revealed {
                return Ok(acc);
            }
            if let Cell::Empty(x) = item.0 {
                if x == 0 {
                    let mut recur_acc = self.reveal_neighbors(player, *c)?;
                    acc.append(&mut recur_acc)
                } else if self.reveal(player, *c) {
                    acc.push(*c)
                }
            } else {
                bail!("Called reveal neighbors when there is a bomb nearby")
            }
            Ok(acc)
        })
    }

    fn plant(&mut self, cell_point: BoardPoint) {
        self.available.remove(&cell_point);

        self.board[cell_point].0 = self.board[cell_point].0.plant().unwrap();

        let neighbors = self.board.neighbors(cell_point);
        neighbors
            .iter()
            .copied()
            .for_each(|c| self.board[c].0 = self.board[c].0.increment());
    }

    fn unplant(&mut self, cell_point: BoardPoint, rem_neighbors: bool) {
        let neighbors = self.board.neighbors(cell_point);

        let was_bomb = self.board[cell_point].0.is_bomb();
        if was_bomb {
            let neighboring_bombs = neighbors
                .iter()
                .copied()
                .fold(0, |acc, c| acc + bool_to_u8(self.board[c].0.is_bomb()));

            self.board[cell_point].0 = self.board[cell_point].0.unplant(neighboring_bombs).unwrap();
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
    flags: Vec<BoardPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    Flag,
    Click,
    DoubleClick,
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

#[cfg(test)]
mod test {
    use crate::board::BoardPoint;
    use crate::cell::Cell;
    use crate::game::{Action, Minesweeper, PlayOutcome};

    const POINT_0_0: BoardPoint = BoardPoint { row: 0, col: 0 };
    const POINT_0_1: BoardPoint = BoardPoint { row: 0, col: 1 };
    const POINT_0_2: BoardPoint = BoardPoint { row: 0, col: 2 };
    const POINT_1_0: BoardPoint = BoardPoint { row: 1, col: 0 };
    const POINT_1_1: BoardPoint = BoardPoint { row: 1, col: 1 };
    const POINT_1_2: BoardPoint = BoardPoint { row: 1, col: 2 };
    const POINT_2_1: BoardPoint = BoardPoint { row: 2, col: 1 };

    fn set_up_game(plant_3_0: bool) -> Minesweeper {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(POINT_0_0);
        game.plant(POINT_1_1);
        game.plant(POINT_1_2);
        if plant_3_0 {
            game.plant(POINT_2_1);
        }
        game
    }

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

        game.plant(POINT_0_0);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 1);
        assert_eq!(game.available.len(), 9 * 9 - 1);
        let first = game.board[POINT_0_0].0;
        assert!(matches!(first, Cell::Bomb));
        let right = game.board[POINT_0_1].0;
        assert!(matches!(right, Cell::Empty(1)));
        let below = game.board[POINT_1_0].0;
        assert!(matches!(below, Cell::Empty(1)));
        let right_below = game.board[POINT_1_1].0;
        assert!(matches!(right_below, Cell::Empty(1)));
        let two_right = game.board[POINT_0_2].0;
        assert!(matches!(two_right, Cell::Empty(0)));

        game.plant(POINT_1_1);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 2);
        assert_eq!(game.available.len(), 9 * 9 - 2);
        let first = game.board[POINT_0_0].0;
        assert!(matches!(first, Cell::Bomb));
        let right = game.board[POINT_0_1].0;
        assert!(matches!(right, Cell::Empty(2)));
        let below = game.board[POINT_1_0].0;
        assert!(matches!(below, Cell::Empty(2)));
        let right_below = game.board[POINT_1_1].0;
        assert!(matches!(right_below, Cell::Bomb));
        let two_right = game.board[POINT_0_2].0;
        assert!(matches!(two_right, Cell::Empty(1)));

        game.plant(POINT_1_2);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 3);
        assert_eq!(game.available.len(), 9 * 9 - 3);
        let first = game.board[POINT_0_0].0;
        assert!(matches!(first, Cell::Bomb));
        let right = game.board[POINT_0_1].0;
        assert!(matches!(right, Cell::Empty(3)));
        let below = game.board[POINT_1_0].0;
        assert!(matches!(below, Cell::Empty(2)));
        let right_below = game.board[POINT_1_1].0;
        assert!(matches!(right_below, Cell::Bomb));
        let two_right = game.board[POINT_0_2].0;
        assert!(matches!(two_right, Cell::Empty(2)));
    }

    #[test]
    fn unplant_bomb_works() {
        let mut game = set_up_game(false);

        game.unplant(POINT_0_0, true);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 1);
        assert_eq!(game.available.len(), 9 * 9 - 3);
        let clicked_bomb_loc = game.board[POINT_0_0].0;
        assert!(matches!(clicked_bomb_loc, Cell::Empty(0)));
        let second_bomb_loc = game.board[POINT_1_1].0;
        assert!(matches!(second_bomb_loc, Cell::Empty(1)));
    }

    #[test]
    fn unplant_cell_works() {
        let mut game = set_up_game(false);

        game.unplant(POINT_0_2, true);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 1);
        assert_eq!(game.available.len(), 9 * 9 - 3);
        let clicked_loc = game.board[POINT_0_2].0;
        assert!(matches!(clicked_loc, Cell::Empty(0)));
        let first_bomb_loc = game.board[POINT_0_0].0;
        assert!(matches!(first_bomb_loc, Cell::Bomb));
        let second_bomb_loc = game.board[POINT_1_1].0;
        assert!(matches!(second_bomb_loc, Cell::Empty(1)));
    }

    #[test]
    fn first_play_bomb_works() {
        let mut game = set_up_game(true);

        let res = game
            .play(0, Action::Click, BoardPoint { row: 0, col: 0 })
            .unwrap();
        assert_eq!(res.len(), 4);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 2);
        assert_eq!(game.available.len(), 9 * 9 - 6);
        let clicked_bomb_loc = &game.board[POINT_0_0];
        assert!(matches!(clicked_bomb_loc.0, Cell::Empty(0)));
        assert!(clicked_bomb_loc.1.revealed);
        assert_eq!(clicked_bomb_loc.1.player, Some(0));
        let second_bomb_loc = &game.board[POINT_1_1];
        assert_eq!(second_bomb_loc.0, Cell::Empty(2));
        assert!(second_bomb_loc.1.revealed);
        assert_eq!(second_bomb_loc.1.player, Some(0));
        let third_bomb_loc = &game.board[POINT_1_2];
        assert!(matches!(third_bomb_loc.0, Cell::Bomb));
        assert!(!third_bomb_loc.1.revealed);
        assert_eq!(third_bomb_loc.1.player, None);
    }

    #[test]
    fn first_play_cell_works() {
        let mut game = set_up_game(true);

        let res = game.play(0, Action::Click, BoardPoint { col: 7, row: 7 });
        assert_eq!(res.unwrap().len(), 9 * 9 - 8);

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 4);
        assert_eq!(game.available.len(), 4); // not bomb and not revealed
        let clicked_loc = &game.board[BoardPoint { row: 8, col: 8 }];
        assert!(matches!(clicked_loc.0, Cell::Empty(0)));
        assert!(clicked_loc.1.revealed);
        assert_eq!(clicked_loc.1.player, Some(0));
        let second_bomb_loc = &game.board[POINT_1_1];
        assert!(matches!(second_bomb_loc.0, Cell::Bomb));
        assert!(!second_bomb_loc.1.revealed);
        assert_eq!(second_bomb_loc.1.player, None);
        let third_bomb_loc = &game.board[POINT_1_2];
        assert!(matches!(third_bomb_loc.0, Cell::Bomb));
        assert!(!third_bomb_loc.1.revealed);
        assert_eq!(third_bomb_loc.1.player, None);
    }

    #[test]
    fn second_click_bomb_failure() {
        let mut game = set_up_game(true);

        let _ = game
            .play(0, Action::Click, BoardPoint { col: 0, row: 0 })
            .unwrap();

        let cell_point = BoardPoint { row: 1, col: 2 };
        let res = game.play(0, Action::Click, cell_point.clone());
        assert!(matches!(res.unwrap(), PlayOutcome::Failure(_)));
    }

    #[test]
    fn second_click_cell_success() {
        let mut game = set_up_game(true);

        let _ = game
            .play(0, Action::Click, BoardPoint { col: 0, row: 0 })
            .unwrap();

        let cell_point = BoardPoint { row: 0, col: 2 };
        let res = game.play(0, Action::Click, cell_point.clone()).unwrap();
        assert!(matches!(res.clone(), PlayOutcome::Success(_)));
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn points_work() {
        let mut game = set_up_game(true);

        let _ = game
            .play(0, Action::Click, BoardPoint { col: 0, row: 0 })
            .unwrap();

        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, 2);

        let cell_point = BoardPoint { row: 0, col: 2 };
        let res = game.play(0, Action::Click, cell_point.clone());
        assert!(matches!(res.unwrap(), PlayOutcome::Success(_)));
    }

    #[test]
    fn dead_errors() {
        let mut game = set_up_game(true);

        let _ = game
            .play(0, Action::Click, BoardPoint { col: 0, row: 0 })
            .unwrap();
        let _ = game
            .play(0, Action::Click, BoardPoint { row: 1, col: 2 })
            .unwrap();

        let res = game.play(0, Action::Click, BoardPoint { row: 3, col: 3 });
        assert!(matches!(res, Err(..)));
    }

    #[test]
    fn revealed_errors() {
        let mut game = set_up_game(true);

        let _ = game
            .play(0, Action::Click, BoardPoint { col: 0, row: 0 })
            .unwrap();

        let res = game.play(0, Action::Click, BoardPoint { row: 1, col: 1 });
        assert!(matches!(res, Err(..)));
    }

    #[test]
    fn oob_errors() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        let res = game.play(0, Action::Click, BoardPoint { col: 10, row: 0 });
        assert!(matches!(res, Err(..)));

        let res = game.play(0, Action::Click, BoardPoint { col: 0, row: 10 });
        assert!(matches!(res, Err(..)));
    }

    #[test]
    fn victory_works() {
        let mut game = set_up_game(true);

        let _ = game
            .play(0, Action::Click, BoardPoint { col: 0, row: 0 })
            .unwrap();
        let _ = game
            .play(0, Action::Click, BoardPoint { row: 8, col: 8 })
            .unwrap();

        let _ = game
            .play(0, Action::Click, BoardPoint { row: 0, col: 2 })
            .unwrap();
        let res = game
            .play(0, Action::Click, BoardPoint { row: 2, col: 0 })
            .unwrap();
        assert!(matches!(res, PlayOutcome::Victory(..)));
        assert_eq!(game.players[0].points, 79);
    }
}
