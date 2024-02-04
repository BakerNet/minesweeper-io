use std::collections::HashSet;

use crate::board::{Board, BoardPoint};
use crate::cell::{Cell, CellState, PlayerCell, RevealedCell};

use anyhow::{bail, Ok, Result};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

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

    pub fn player_score(&self, player: usize) -> Result<usize> {
        if player > self.players.len() - 1 {
            bail!("Player {player} doesn't exist")
        }
        Ok(self.players[player].score)
    }

    pub fn player_dead(&self, player: usize) -> Result<bool> {
        if player > self.players.len() - 1 {
            bail!("Player {player} doesn't exist")
        }
        Ok(self.players[player].dead)
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
            Action::Reveal => self.handle_click(player, cell_point),
            Action::RevealAdjacent => self.handle_double_click(player, cell_point),
            Action::Flag => self.handle_flag(player, cell_point),
        }
    }

    fn handle_flag(&mut self, player: usize, cell_point: BoardPoint) -> Result<PlayOutcome> {
        let (_, cell_state) = &self.board[cell_point];
        if cell_state.revealed {
            bail!("Tried to play already revealed cell")
        }
        let player_cell = if self.players[player].flags.contains(&cell_point) {
            self.players[player].flags.remove(&cell_point);
            PlayerCell::Hidden
        } else {
            self.players[player].flags.insert(cell_point);
            PlayerCell::Flag
        };
        Ok(PlayOutcome::Flag((cell_point, player_cell)))
    }

    fn handle_click(&mut self, player: usize, cell_point: BoardPoint) -> Result<PlayOutcome> {
        let (_, cell_state) = &self.board[cell_point];
        if cell_state.revealed {
            bail!("Tried to play already revealed cell")
        }
        if self.players[player].flags.contains(&cell_point) {
            bail!("Tried to play flagged cell")
        }
        if !(self.players[player].played) && self.has_no_revealed_neighbors(cell_point) {
            self.players[player].played = true;
            self.unplant(cell_point, true);
        }
        let (cell, _) = &self.board[cell_point];
        match cell {
            Cell::Bomb => {
                self.reveal(player, cell_point);
                self.players[player].dead = true;
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
                self.players[player].score += revealed_points.len();
                if self.available.is_empty() {
                    Ok(PlayOutcome::Victory(revealed_points))
                } else {
                    Ok(PlayOutcome::Success(revealed_points))
                }
            }
            Cell::Empty(_) => {
                self.reveal(player, cell_point);
                self.players[player].score += 1;
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
        let (cell, cell_state) = &self.board[cell_point];
        if !cell_state.revealed {
            bail!("Tried to double-click cell that isn't revealed")
        }
        let neighbors = self.board.neighbors(cell_point);
        let flagged_neighbors = neighbors
            .iter()
            .copied()
            .filter(|c| self.players[player].flags.contains(c) || self.is_revealed_bomb(*c));
        if let Cell::Empty(x) = cell {
            if *x == 0 {
                bail!("Tried to double-click zero space")
            }
            let flagged_count = flagged_neighbors.count();
            if *x as usize != flagged_count {
                bail!("Tried to double-click with wrong number of flagged neighbors.  Expected {x} got {flagged_count}")
            }
        } else {
            bail!("Tried to double-click bomb")
        }
        let unflagged_neighbors = neighbors
            .iter()
            .copied()
            .filter(|c| !self.board[*c].1.revealed && !self.players[player].flags.contains(c))
            .collect::<Vec<_>>();
        let has_bomb = unflagged_neighbors
            .iter()
            .copied()
            .find(|c| matches!(self.board[*c].0, Cell::Bomb));
        // check for bomb first, so other clicks don't go through
        if let Some(c) = has_bomb {
            self.reveal(player, c);
            self.players[player].dead = true;
            return Ok(PlayOutcome::Failure(RevealedCell {
                cell_point: c,
                player,
                contents: self.board[cell_point].0,
            }));
        }
        let combined_outcome = unflagged_neighbors.iter().fold(
            PlayOutcome::Success(Vec::new()),
            |acc: PlayOutcome, c| {
                if self.board[*c].1.revealed {
                    return acc;
                }
                let res = self
                    .handle_click(player, *c)
                    .expect("Handle click inside double-click should work");
                acc.combine(res)
            },
        );
        Ok(combined_outcome)
    }

    fn is_revealed_bomb(&self, cell_point: BoardPoint) -> bool {
        let item = self.board[cell_point];
        item.1.revealed && item.0.is_bomb()
    }

    pub fn is_over(&self) -> bool {
        self.available.is_empty() || self.players.iter().all(|x| x.dead)
    }

    pub fn viewer_board(&self) -> Vec<Vec<PlayerCell>> {
        let mut return_board: Vec<Vec<PlayerCell>> =
            vec![vec![PlayerCell::Hidden; self.board.cols()]; self.board.rows()];
        for (r_num, row) in return_board.iter_mut().enumerate() {
            for (c_num, return_item) in row.iter_mut().enumerate() {
                let point = BoardPoint {
                    row: r_num,
                    col: c_num,
                };
                let item = &self.board[point];
                if item.1.revealed {
                    *return_item = PlayerCell::Revealed(RevealedCell {
                        cell_point: point,
                        player: item.1.player.unwrap(),
                        contents: item.0,
                    });
                }
            }
        }
        return_board
    }

    pub fn player_board(&self, player: usize) -> Vec<Vec<PlayerCell>> {
        let mut return_board = self.viewer_board();
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
            self.players.iter_mut().for_each(|p| {
                p.flags.remove(&cell_point);
            });
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
            let item = self.board[*c];
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

    fn has_no_revealed_neighbors(&self, cell_point: BoardPoint) -> bool {
        let neighbors = self.board.neighbors(cell_point);
        neighbors
            .iter()
            .copied()
            .filter(|i| self.board[*i].1.revealed)
            .count()
            == 0
    }

    fn plant(&mut self, cell_point: BoardPoint) {
        self.available.remove(&cell_point);

        self.board[cell_point].0 = self.board[cell_point].0.plant().unwrap();

        let neighbors = self.board.neighbors(cell_point);
        neighbors.iter().copied().for_each(|c| {
            self.board[c].0 = self.board[c].0.increment();
        });
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

        // TODO - implement replanting mines to keep num_mines intact
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
    score: usize,
    flags: HashSet<BoardPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    Flag,
    Reveal,
    RevealAdjacent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayOutcome {
    Success(Vec<RevealedCell>),
    Failure(RevealedCell),
    Victory(Vec<RevealedCell>),
    Flag((BoardPoint, PlayerCell)),
}

impl PlayOutcome {
    pub fn len(&self) -> usize {
        match self {
            Self::Success(v) => v.len(),
            Self::Victory(v) => v.len(),
            Self::Failure(_) => 1,
            Self::Flag(_) => 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn combine(self, other: PlayOutcome) -> Self {
        let mut is_victory = false;
        let mut vec = match self {
            PlayOutcome::Success(x) => x,
            PlayOutcome::Victory(x) => {
                is_victory = true;
                x
            }
            PlayOutcome::Failure(_) => {
                return self;
            }
            PlayOutcome::Flag(_) => {
                return self;
            }
        };
        match other {
            PlayOutcome::Failure(_) => other,
            PlayOutcome::Flag(_) => other, // this shouldn't happen
            PlayOutcome::Success(mut x) => {
                vec.append(&mut x);
                if is_victory {
                    PlayOutcome::Victory(vec)
                } else {
                    PlayOutcome::Success(vec)
                }
            }
            PlayOutcome::Victory(mut x) => {
                vec.append(&mut x);
                PlayOutcome::Victory(vec)
            }
        }
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

    fn num_bombs(game: &Minesweeper, number: usize) {
        let num_bombs = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Bomb))
            .count();
        assert_eq!(num_bombs, number);
    }

    fn point_cell(game: &Minesweeper, point: BoardPoint, _cell: Cell) {
        let board_cell = game.board[point].0;
        assert!(matches!(board_cell, _cell));
    }

    fn point_cell_state(
        game: &Minesweeper,
        point: BoardPoint,
        revealed: bool,
        player: Option<usize>,
    ) {
        let board_cell_state = game.board[point].1;
        assert_eq!(board_cell_state.revealed, revealed);
        assert_eq!(board_cell_state.player, player);
    }

    #[test]
    fn create_and_init_game() {
        let game = Minesweeper::init_game(9, 9, 10, 1).unwrap();
        num_bombs(&game, 10);
    }

    #[test]
    fn plant_works() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        game.plant(POINT_0_0);

        num_bombs(&game, 1);
        assert_eq!(game.available.len(), 9 * 9 - 1);
        point_cell(&game, POINT_0_0, Cell::Bomb);
        point_cell(&game, POINT_0_1, Cell::Empty(1));
        point_cell(&game, POINT_1_0, Cell::Empty(1));
        point_cell(&game, POINT_1_1, Cell::Empty(1));
        point_cell(&game, POINT_0_2, Cell::Empty(0));

        game.plant(POINT_1_1);

        num_bombs(&game, 2);
        assert_eq!(game.available.len(), 9 * 9 - 2);
        point_cell(&game, POINT_0_0, Cell::Bomb);
        point_cell(&game, POINT_0_1, Cell::Empty(2));
        point_cell(&game, POINT_1_0, Cell::Empty(2));
        point_cell(&game, POINT_1_1, Cell::Bomb);
        point_cell(&game, POINT_0_2, Cell::Empty(1));

        game.plant(POINT_1_2);

        num_bombs(&game, 3);
        assert_eq!(game.available.len(), 9 * 9 - 3);
        point_cell(&game, POINT_0_0, Cell::Bomb);
        point_cell(&game, POINT_0_1, Cell::Empty(3));
        point_cell(&game, POINT_1_0, Cell::Empty(2));
        point_cell(&game, POINT_1_1, Cell::Bomb);
        point_cell(&game, POINT_0_2, Cell::Empty(2));
    }

    #[test]
    fn unplant_bomb_works() {
        let mut game = set_up_game(false);

        game.unplant(POINT_0_0, true);

        num_bombs(&game, 1);
        assert_eq!(game.available.len(), 9 * 9 - 3);
        point_cell(&game, POINT_0_0, Cell::Empty(0));
        point_cell(&game, POINT_1_1, Cell::Empty(1));
    }

    #[test]
    fn unplant_cell_works() {
        let mut game = set_up_game(false);

        game.unplant(POINT_0_2, true);

        num_bombs(&game, 1);
        assert_eq!(game.available.len(), 9 * 9 - 3);
        point_cell(&game, POINT_0_2, Cell::Empty(0));
        point_cell(&game, POINT_0_0, Cell::Bomb);
        point_cell(&game, POINT_1_1, Cell::Empty(1));
    }

    #[test]
    fn first_play_bomb_works() {
        let mut game = set_up_game(true);

        let res = game
            .play(0, Action::Reveal, BoardPoint { row: 0, col: 0 })
            .unwrap();
        assert_eq!(res.len(), 4);

        num_bombs(&game, 2);
        assert_eq!(game.available.len(), 9 * 9 - 6);
        point_cell(&game, POINT_0_0, Cell::Empty(0));
        point_cell_state(&game, POINT_0_0, true, Some(0));
        point_cell(&game, POINT_1_1, Cell::Empty(2));
        point_cell_state(&game, POINT_1_1, true, Some(0));
        point_cell(&game, POINT_1_2, Cell::Bomb);
        point_cell_state(&game, POINT_1_2, false, None);
    }

    #[test]
    fn first_play_cell_works() {
        let mut game = set_up_game(true);

        let res = game.play(0, Action::Reveal, BoardPoint { col: 7, row: 7 });
        assert_eq!(res.unwrap().len(), 9 * 9 - 8);

        num_bombs(&game, 4);
        assert_eq!(game.available.len(), 4); // not bomb and not revealed
        point_cell(&game, BoardPoint { row: 8, col: 8 }, Cell::Empty(0));
        point_cell_state(&game, BoardPoint { row: 8, col: 8 }, true, Some(0));
        point_cell(&game, POINT_1_1, Cell::Bomb);
        point_cell_state(&game, POINT_1_1, false, None);
        point_cell(&game, POINT_1_2, Cell::Bomb);
        point_cell_state(&game, POINT_1_2, false, None);
    }

    #[test]
    fn second_click_bomb_failure() {
        let mut game = set_up_game(true);

        let _ = game.play(0, Action::Reveal, POINT_0_0).unwrap();

        let cell_point = BoardPoint { row: 1, col: 2 };
        let res = game.play(0, Action::Reveal, cell_point.clone());
        assert!(matches!(res.unwrap(), PlayOutcome::Failure(_)));
    }

    #[test]
    fn second_click_cell_success() {
        let mut game = set_up_game(true);

        let _ = game.play(0, Action::Reveal, POINT_0_0).unwrap();

        let cell_point = BoardPoint { row: 0, col: 2 };
        let res = game.play(0, Action::Reveal, cell_point.clone()).unwrap();
        assert!(matches!(res.clone(), PlayOutcome::Success(_)));
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn flag_works() {
        let mut game = set_up_game(true);

        let _ = game.play(0, Action::Reveal, POINT_0_0).unwrap();

        let cell_point = BoardPoint { row: 1, col: 2 };
        let res = game.play(0, Action::Flag, cell_point.clone()).unwrap();
        assert!(matches!(res, PlayOutcome::Flag(_)));

        let res = game.play(0, Action::Reveal, cell_point.clone());
        assert!(matches!(res, Err(_)));
    }

    #[test]
    fn unflag_works() {
        let mut game = set_up_game(true);

        let _ = game.play(0, Action::Reveal, POINT_0_0).unwrap();

        let cell_point = BoardPoint { row: 1, col: 2 };
        let _ = game.play(0, Action::Flag, cell_point.clone()).unwrap();
        let res = game.play(0, Action::Flag, cell_point.clone()).unwrap();
        assert!(matches!(res, PlayOutcome::Flag(_)));

        let res = game.play(0, Action::Reveal, cell_point.clone()).unwrap();
        assert!(matches!(res, PlayOutcome::Failure(_)));
    }

    #[test]
    fn double_click_works() {
        let mut game = set_up_game(true);

        let res = game
            .play(0, Action::Reveal, BoardPoint { row: 0, col: 0 })
            .unwrap();
        assert_eq!(res.len(), 4);

        num_bombs(&game, 2);

        let _ = game.play(0, Action::Flag, POINT_1_2).unwrap();
        let _ = game.play(0, Action::Flag, POINT_2_1).unwrap();
        let _ = game
            .play(0, Action::Reveal, BoardPoint { row: 2, col: 2 })
            .unwrap();
        point_cell(&game, BoardPoint { row: 2, col: 2 }, Cell::Empty(2));

        let res = game
            .play(0, Action::RevealAdjacent, BoardPoint { row: 2, col: 2 })
            .expect("double-click should work");
        assert_eq!(res.len(), 9 * 9 - 9);
    }

    #[test]
    fn bad_double_click_fails() {
        let mut game = set_up_game(true);

        let res = game
            .play(0, Action::Reveal, BoardPoint { row: 0, col: 0 })
            .unwrap();
        assert_eq!(res.len(), 4);

        num_bombs(&game, 2);

        let _ = game.play(0, Action::Flag, POINT_1_2).unwrap();
        let _ = game
            .play(0, Action::Reveal, BoardPoint { row: 2, col: 2 })
            .unwrap();
        point_cell(&game, BoardPoint { row: 2, col: 2 }, Cell::Empty(2));

        let res = game.play(0, Action::RevealAdjacent, BoardPoint { row: 2, col: 2 });
        assert!(matches!(res, Err(_)));
    }

    #[test]
    fn score_works() {
        let mut game = set_up_game(true);

        let _ = game.play(0, Action::Reveal, POINT_0_0).unwrap();

        num_bombs(&game, 2);

        let cell_point = BoardPoint { row: 0, col: 2 };
        let res = game.play(0, Action::Reveal, cell_point.clone());
        assert!(matches!(res.unwrap(), PlayOutcome::Success(_)));
        assert_eq!(game.players[0].score, 5);
    }

    #[test]
    fn dead_errors() {
        let mut game = set_up_game(true);

        let _ = game.play(0, Action::Reveal, POINT_0_0).unwrap();
        let _ = game
            .play(0, Action::Reveal, BoardPoint { row: 1, col: 2 })
            .unwrap();

        let res = game.play(0, Action::Reveal, BoardPoint { row: 3, col: 3 });
        assert!(matches!(res, Err(..)));
    }

    #[test]
    fn revealed_errors() {
        let mut game = set_up_game(true);

        let _ = game.play(0, Action::Reveal, POINT_0_0).unwrap();

        let res = game.play(0, Action::Reveal, BoardPoint { row: 1, col: 1 });
        assert!(matches!(res, Err(..)));
    }

    #[test]
    fn oob_errors() {
        let mut game = Minesweeper::new(9, 9, 10, 1).unwrap();

        let res = game.play(0, Action::Reveal, BoardPoint { col: 10, row: 0 });
        assert!(matches!(res, Err(..)));

        let res = game.play(0, Action::Reveal, BoardPoint { col: 0, row: 10 });
        assert!(matches!(res, Err(..)));
    }

    #[test]
    fn victory_works() {
        let mut game = set_up_game(true);

        let _ = game.play(0, Action::Reveal, POINT_0_0).unwrap();
        let _ = game
            .play(0, Action::Reveal, BoardPoint { row: 8, col: 8 })
            .unwrap();

        let _ = game
            .play(0, Action::Reveal, BoardPoint { row: 0, col: 2 })
            .unwrap();
        let res = game
            .play(0, Action::Reveal, BoardPoint { row: 2, col: 0 })
            .unwrap();
        assert!(matches!(res, PlayOutcome::Victory(..)));
        assert_eq!(game.players[0].score, 79);
    }
}
