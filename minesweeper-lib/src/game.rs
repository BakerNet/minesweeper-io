use std::cmp::max;
use std::collections::HashSet;

use crate::board::{Board, BoardPoint};
use crate::cell::{Cell, CellState, HiddenCell, PlayerCell, RevealedCell};
use crate::client::ClientPlayer;
use crate::replay::MinesweeperReplay;

use anyhow::{bail, Ok, Result};
use rand::{seq::SliceRandom, thread_rng};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug)]
pub struct MinesweeperOpts {
    pub rows: usize,
    pub cols: usize,
    pub num_mines: usize,
}

impl MinesweeperOpts {
    fn validate(&self) -> bool {
        if self.rows == 0 || self.cols == 0 || self.num_mines == 0 {
            return false;
        }
        let total = self.rows * self.cols;
        self.num_mines < total
    }
}

pub struct MinesweeperBuilder {
    opts: MinesweeperOpts,
    players: Option<usize>,
    log: bool,
    superclick: bool,
}

impl MinesweeperBuilder {
    pub fn new(opts: MinesweeperOpts) -> Result<Self> {
        if !opts.validate() {
            bail!("Invalid minesweeper options")
        }
        Ok(Self {
            opts,
            players: None,
            log: false,
            superclick: false,
        })
    }

    pub fn with_multiplayer(mut self, players: usize) -> Self {
        self.players = Some(players);
        self
    }

    pub fn with_log(mut self) -> Self {
        self.log = true;
        self
    }

    pub fn with_superclick(mut self) -> Self {
        self.superclick = true;
        self
    }

    pub fn init(self) -> Minesweeper {
        let mut board = Board::new(
            self.opts.rows,
            self.opts.cols,
            (Cell::default(), CellState::default()),
        );
        let mut available: Vec<_> = (0..board.len())
            .map(|x| board.point_from_index(x))
            .collect();
        available.shuffle(&mut thread_rng());
        let points_to_plant = &available[0..self.opts.num_mines];
        points_to_plant.iter().for_each(|&x| {
            board[x].0 = board[x].0.plant().unwrap();

            let neighbors = board.neighbors(x);
            neighbors.into_iter().for_each(|c| {
                board[c].0 = board[c].0.increment();
            });
        });
        let available = available.into_iter().skip(self.opts.num_mines).collect();
        Minesweeper {
            available,
            players: vec![Player::default(); self.players.unwrap_or(1)],
            board,
            superclick: self.superclick,
            log: if self.log { Some(Vec::new()) } else { None },
        }
    }
}

impl Board<(Cell, CellState)> {
    fn viewer_board(&self, is_final: bool) -> Board<PlayerCell> {
        let mut new_board =
            Board::<PlayerCell>::new(self.rows(), self.cols(), PlayerCell::default());
        for row in 0..self.rows() {
            for col in 0..self.cols() {
                let point = BoardPoint { row, col };
                let item = &self[point];
                if item.1.revealed {
                    new_board[point] = PlayerCell::Revealed(RevealedCell {
                        player: item.1.player.unwrap(),
                        contents: item.0,
                    });
                } else if is_final && matches!(item.0, Cell::Mine) {
                    new_board[point] = PlayerCell::Hidden(HiddenCell::Mine)
                }
            }
        }
        new_board
    }

    #[allow(dead_code)]
    fn player_board(&self, player_flags: HashSet<BoardPoint>, is_final: bool) -> Board<PlayerCell> {
        let mut new_board =
            Board::<PlayerCell>::new(self.rows(), self.cols(), PlayerCell::default());
        for row in 0..self.rows() {
            for col in 0..self.cols() {
                let point = BoardPoint { row, col };
                let item = &self[point];
                if item.1.revealed {
                    new_board[point] = PlayerCell::Revealed(RevealedCell {
                        player: item.1.player.unwrap(),
                        contents: item.0,
                    });
                } else if is_final && matches!(item.0, Cell::Mine) {
                    new_board[point] = PlayerCell::Hidden(HiddenCell::Mine)
                }
                if player_flags.contains(&point) {
                    new_board[point] = new_board[point].add_flag()
                }
            }
        }
        new_board
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct Play {
    #[serde(rename = "p", alias = "player")]
    pub player: usize,
    #[serde(rename = "a", alias = "action")]
    pub action: Action,
    #[serde(rename = "bp", alias = "point")]
    pub point: BoardPoint,
}

pub struct Minesweeper {
    available: HashSet<BoardPoint>,
    players: Vec<Player>,
    board: Board<(Cell, CellState)>,
    log: Option<Vec<(Play, PlayOutcome)>>,
    superclick: bool,
}

impl Minesweeper {
    fn handle_flag(&mut self, player: usize, cell_point: BoardPoint) -> Result<PlayOutcome> {
        let (_, cell_state) = &self.board[cell_point];
        if cell_state.revealed {
            bail!("Tried to play already revealed cell")
        }
        let player_cell = if self.players[player].flags.contains(&cell_point) {
            self.players[player].flags.remove(&cell_point);
            PlayerCell::Hidden(HiddenCell::Empty)
        } else {
            self.players[player].flags.insert(cell_point);
            PlayerCell::Hidden(HiddenCell::Flag)
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
        let mut update_revealed = None::<Vec<BoardPoint>>;
        if !(self.players[player].played) && self.has_no_revealed_nearby(cell_point) {
            // on first click of empty board space, prevent mine
            self.players[player].played = true;
            update_revealed = Some(self.unplant(cell_point, self.superclick));
        }
        let (cell, _) = &self.board[cell_point];
        match cell {
            Cell::Mine => {
                self.reveal(player, cell_point);
                self.players[player].dead = true;
                Ok(PlayOutcome::Failure((
                    cell_point,
                    RevealedCell {
                        player,
                        contents: self.board[cell_point].0,
                    },
                )))
            }
            Cell::Empty(x) if x == &0 => {
                let mut revealed_points = self.reveal_neighbors(player, cell_point)?;
                if let Some(updated_points) = update_revealed {
                    revealed_points.extend(updated_points);
                }
                let revealed_points = revealed_points
                    .into_iter()
                    .map(|c| {
                        (
                            c,
                            RevealedCell {
                                player,
                                contents: self.board[c].0,
                            },
                        )
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
                let revealed_point = vec![(
                    cell_point,
                    RevealedCell {
                        player,
                        contents: self.board[cell_point].0,
                    },
                )];
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
            .filter(|c| self.players[player].flags.contains(c) || self.is_revealed_mine(*c));
        if let Cell::Empty(x) = cell {
            if *x == 0 {
                bail!("Tried to double-click zero space")
            }
            let flagged_count = flagged_neighbors.count();
            if *x as usize != flagged_count {
                bail!("Tried to double-click with wrong number of flagged neighbors.  Expected {x} got {flagged_count}")
            }
        } else {
            bail!("Tried to double-click mine")
        }
        let unflagged_neighbors = neighbors
            .iter()
            .copied()
            .filter(|c| !self.board[*c].1.revealed && !self.players[player].flags.contains(c))
            .collect::<Vec<_>>();
        let has_mine = unflagged_neighbors
            .iter()
            .copied()
            .find(|c| matches!(self.board[*c].0, Cell::Mine));
        // check for mine first, so other clicks don't go through
        if let Some(c) = has_mine {
            self.reveal(player, c);
            self.players[player].dead = true;
            return Ok(PlayOutcome::Failure((
                c,
                RevealedCell {
                    player,
                    contents: self.board[c].0,
                },
            )));
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

    fn is_revealed_mine(&self, cell_point: BoardPoint) -> bool {
        let item = self.board[cell_point];
        item.1.revealed && item.0.is_mine()
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
                bail!("Called reveal neighbors when there is a mine nearby")
            }
            Ok(acc)
        })
    }

    fn has_no_revealed_nearby(&self, cell_point: BoardPoint) -> bool {
        let neighbors = self.board.neighbors(cell_point);
        let nearby = neighbors
            .into_iter()
            .flat_map(|n| self.board.neighbors(n))
            .collect::<HashSet<_>>();
        nearby
            .into_iter()
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

    fn unplant(&mut self, cell_point: BoardPoint, rem_neighbors: bool) -> Vec<BoardPoint> {
        let mut updated_revealed = HashSet::new();
        let mut to_replant = if rem_neighbors { Some(0) } else { None };

        let neighbors = self.board.neighbors(cell_point);

        let was_mine = self.board[cell_point].0.is_mine();
        if was_mine {
            let neighboring_mines = neighbors
                .iter()
                .copied()
                .fold(0, |acc, c| acc + bool_to_u8(self.board[c].0.is_mine()));

            // set value to number of neighboring mine
            self.board[cell_point].0 = self.board[cell_point].0.unplant(neighboring_mines).unwrap();

            if rem_neighbors {
                if let Some(unplanted_mines) = &mut to_replant {
                    *unplanted_mines += 1;
                }
            }
        }

        neighbors.iter().copied().for_each(|i| {
            let new = if was_mine {
                if self.board[i].1.revealed {
                    updated_revealed.insert(i);
                }
                self.board[i].0.decrement()
            } else {
                self.board[i].0
            };
            if rem_neighbors && matches!(new, Cell::Mine) {
                updated_revealed.extend(self.unplant(i, false));
                if let Some(unplanted_mines) = &mut to_replant {
                    *unplanted_mines += 1;
                }
            } else {
                self.board[i].0 = new;
            }
        });

        if let Some(unplanted_mines) = to_replant {
            self.replant(unplanted_mines, cell_point, neighbors);
        }

        updated_revealed.into_iter().collect()
    }

    fn replant(
        &mut self,
        unplanted_mines: usize,
        first_cell: BoardPoint,
        neighbors: Vec<BoardPoint>,
    ) {
        if unplanted_mines == 0 {
            return;
        }
        let has_revealed_neighbor = |bp: BoardPoint| {
            self.board
                .neighbors(bp)
                .iter()
                .any(|c| self.board[*c].1.revealed)
        };
        let mut take_available: Vec<BoardPoint> = self
            .available
            .iter()
            .filter(|&bp| {
                *bp != first_cell && !neighbors.contains(bp) && !has_revealed_neighbor(*bp)
            })
            .copied()
            .collect::<Vec<_>>();
        let mut rng = thread_rng();
        take_available.shuffle(&mut rng);
        if unplanted_mines > take_available.len() {
            let mut unplanted_points = neighbors;
            unplanted_points.shuffle(&mut rng);
            take_available.extend(unplanted_points);
        }
        take_available.iter().take(unplanted_mines).for_each(|x| {
            self.plant(*x);
        });
    }
}

impl Minesweeper {
    pub fn complete(self) -> CompletedMinesweeper {
        CompletedMinesweeper {
            players: self.players,
            board: self.board.viewer_board(true),
            log: self.log,
        }
    }

    pub fn play(&mut self, play: Play) -> Result<PlayOutcome> {
        if self.is_over() {
            bail!("Game is over")
        }
        if self.players[play.player].dead {
            bail!("Tried to play as dead player")
        }
        if !self.board.is_in_bounds(play.point) {
            bail!("Tried to play point outside of playzone")
        }
        let play_res = match play.action {
            Action::Reveal => self.handle_click(play.player, play.point),
            Action::RevealAdjacent => self.handle_double_click(play.player, play.point),
            Action::Flag => self.handle_flag(play.player, play.point),
        };
        if self.available.is_empty() {
            // game is over
            self.players[play.player].victory_click = true;
        }
        // record play if applicable
        let _ = play_res.as_ref().map(|outcome| {
            if let Some(history) = &mut self.log {
                history.push((play, outcome.clone()));
            }
        });
        play_res
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

    pub fn current_top_score(&self) -> Option<usize> {
        if self.players.len() < 2 {
            None
        } else {
            let top_score = self.players.iter().fold(0, |acc, p| max(p.score, acc));
            match top_score {
                0 => None,
                score => Some(score),
            }
        }
    }

    pub fn player_top_score(&self, player: usize) -> Result<bool> {
        if player > self.players.len() - 1 {
            bail!("Player {player} doesn't exist")
        }
        if self.players.len() < 2 {
            Ok(false) // no top_score in single player
        } else {
            let top_score = self.players.iter().fold(0, |acc, p| max(p.score, acc));
            Ok(self.players[player].score == top_score && top_score != 0)
        }
    }

    pub fn player_victory_click(&self, player: usize) -> Result<bool> {
        if player > self.players.len() - 1 {
            bail!("Player {player} doesn't exist")
        }
        Ok(self.players[player].victory_click)
    }

    pub fn is_over(&self) -> bool {
        self.available.is_empty() || self.players.iter().all(|x| x.dead)
    }

    pub fn viewer_board(&self) -> Vec<Vec<PlayerCell>> {
        self.board.viewer_board(false).into()
    }

    pub fn player_board(&self, player: usize) -> Vec<Vec<PlayerCell>> {
        let mut return_board = self.viewer_board();
        for f in self.players[player].flags.iter() {
            if let PlayerCell::Hidden(_) = return_board[f.row][f.col] {
                return_board[f.row][f.col] = return_board[f.row][f.col].add_flag()
            }
        }
        return_board
    }
}

pub struct CompletedMinesweeper {
    players: Vec<Player>,
    board: Board<PlayerCell>,
    log: Option<Vec<(Play, PlayOutcome)>>,
}

impl CompletedMinesweeper {
    pub fn from_log(
        board: Board<PlayerCell>,
        log: Vec<(Play, PlayOutcome)>,
        players: Vec<ClientPlayer>,
    ) -> CompletedMinesweeper {
        let players_len = players.len();
        let mut players =
            players
                .into_iter()
                .fold(vec![Player::default(); players_len], |mut acc, p| {
                    acc[p.player_id].score = p.score;
                    acc[p.player_id].dead = p.dead;
                    acc[p.player_id].victory_click = p.victory_click;
                    acc
                });
        log.iter()
            .filter(|item| matches!(item.1, PlayOutcome::Flag(_)))
            .for_each(|item| {
                if let PlayOutcome::Flag((point, _)) = item.1 {
                    players[item.0.player].flags.insert(point);
                }
            });
        CompletedMinesweeper {
            players,
            board,
            log: Some(log),
        }
    }

    pub fn recover_log(self) -> Option<Vec<(Play, PlayOutcome)>> {
        self.log
    }
}

impl CompletedMinesweeper {
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

    pub fn player_victory_click(&self, player: usize) -> Result<bool> {
        if player > self.players.len() - 1 {
            bail!("Player {player} doesn't exist")
        }
        Ok(self.players[player].victory_click)
    }

    pub fn top_score(&self) -> Option<usize> {
        if self.players.len() < 2 {
            None
        } else {
            let top_score = self.players.iter().fold(0, |acc, p| max(p.score, acc));
            match top_score {
                0 => None,
                score => Some(score),
            }
        }
    }

    pub fn player_top_score(&self, player: usize) -> Result<bool> {
        if player > self.players.len() - 1 {
            bail!("Player {player} doesn't exist")
        }
        if self.players.len() < 2 {
            Ok(false) // no top_score in single player
        } else {
            let top_score = self.players.iter().fold(0, |acc, p| max(p.score, acc));
            Ok(self.players[player].score == top_score && top_score != 0)
        }
    }

    pub fn viewer_board_final(&self) -> Vec<Vec<PlayerCell>> {
        (&self.board).into()
    }

    pub fn player_board_final(&self, player: usize) -> Vec<Vec<PlayerCell>> {
        let mut return_board = self.viewer_board_final();
        for f in self.players[player].flags.iter() {
            if let PlayerCell::Hidden(_) = return_board[f.row][f.col] {
                return_board[f.row][f.col] = return_board[f.row][f.col].add_flag()
            }
        }
        return_board
    }

    fn board_start(&self) -> Board<PlayerCell> {
        let mut board = self.board.clone();
        board.iter_mut().for_each(|pc| *pc = pc.into_hidden());
        board
    }

    pub fn get_log(&self) -> Option<Vec<(Play, PlayOutcome)>> {
        Some(self.log.as_ref()?.clone())
    }

    pub fn replay(&self, player: Option<usize>) -> Option<MinesweeperReplay> {
        let player_log = self
            .log
            .as_ref()?
            .iter()
            .filter(|po| match po.0.action {
                Action::Flag => Some(po.0.player) == player,
                _ => true,
            })
            .cloned()
            .collect();
        Some(MinesweeperReplay::new(
            self.board_start(),
            player_log,
            self.players.len(),
        ))
    }
}

fn bool_to_u8(b: bool) -> u8 {
    match b {
        true => 1,
        false => 0,
    }
}

#[derive(Clone, Debug, Default)]
pub struct Player {
    played: bool,
    dead: bool,
    victory_click: bool,
    score: usize,
    flags: HashSet<BoardPoint>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    #[serde(rename = "f", alias = "Flag")]
    Flag,
    #[serde(rename = "r", alias = "Reveal")]
    Reveal,
    #[serde(rename = "ra", alias = "RevealAdjacent")]
    RevealAdjacent,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayOutcome {
    #[serde(rename = "s", alias = "Success")]
    Success(Vec<(BoardPoint, RevealedCell)>),
    #[serde(rename = "x", alias = "Failure")]
    Failure((BoardPoint, RevealedCell)),
    #[serde(rename = "v", alias = "Victory")]
    Victory(Vec<(BoardPoint, RevealedCell)>),
    #[serde(rename = "f", alias = "Flag")]
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
    use crate::board::{Board, BoardPoint};
    use crate::cell::{Cell, CellState};

    use super::*;

    const POINT_0_0: BoardPoint = BoardPoint { row: 0, col: 0 };
    const POINT_0_1: BoardPoint = BoardPoint { row: 0, col: 1 };
    const POINT_0_2: BoardPoint = BoardPoint { row: 0, col: 2 };
    const POINT_0_3: BoardPoint = BoardPoint { row: 0, col: 3 };
    const POINT_1_0: BoardPoint = BoardPoint { row: 1, col: 0 };
    const POINT_1_1: BoardPoint = BoardPoint { row: 1, col: 1 };
    const POINT_1_2: BoardPoint = BoardPoint { row: 1, col: 2 };
    const POINT_2_1: BoardPoint = BoardPoint { row: 2, col: 1 };
    const POINT_2_2: BoardPoint = BoardPoint { row: 2, col: 2 };
    const POINT_2_3: BoardPoint = BoardPoint { row: 2, col: 3 };
    const POINT_3_2: BoardPoint = BoardPoint { row: 3, col: 2 };
    const POINT_3_3: BoardPoint = BoardPoint { row: 3, col: 3 };

    fn empty_game(player_num: usize) -> Minesweeper {
        let board = Board::new(9, 9, (Cell::default(), CellState::default()));
        let available = (0..81).map(|x| board.point_from_index(x)).collect();
        Minesweeper {
            available,
            players: vec![Player::default(); player_num],
            board,
            log: None,
            superclick: true,
        }
    }

    fn set_up_game() -> Minesweeper {
        let mut game = empty_game(2);

        game.plant(POINT_0_0);
        game.plant(POINT_1_1);
        game.plant(POINT_1_2);
        game.plant(POINT_2_1);
        game
    }

    fn set_up_game_no_superclick() -> Minesweeper {
        let mut game = set_up_game();
        game.superclick = false;

        game
    }

    fn num_mines(game: &Minesweeper, number: usize) {
        let num_mines = game
            .board
            .iter()
            .filter(|x| matches!(x.0, Cell::Mine))
            .count();
        assert_eq!(num_mines, number);
    }

    fn assert_point_cell(game: &Minesweeper, point: BoardPoint, _cell: Cell) {
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
        let game: Minesweeper = MinesweeperBuilder::new(MinesweeperOpts {
            rows: 9,
            cols: 9,
            num_mines: 10,
        })
        .unwrap()
        .init();
        num_mines(&game, 10);
    }

    #[test]
    fn plant_works() {
        let mut game = empty_game(2);

        game.plant(POINT_0_0);

        num_mines(&game, 1);
        assert_eq!(game.available.len(), 9 * 9 - 1);
        assert_point_cell(&game, POINT_0_0, Cell::Mine);
        assert_point_cell(&game, POINT_0_1, Cell::Empty(1));
        assert_point_cell(&game, POINT_1_0, Cell::Empty(1));
        assert_point_cell(&game, POINT_1_1, Cell::Empty(1));
        assert_point_cell(&game, POINT_0_2, Cell::Empty(0));

        game.plant(POINT_1_1);

        num_mines(&game, 2);
        assert_eq!(game.available.len(), 9 * 9 - 2);
        assert_point_cell(&game, POINT_0_0, Cell::Mine);
        assert_point_cell(&game, POINT_0_1, Cell::Empty(2));
        assert_point_cell(&game, POINT_1_0, Cell::Empty(2));
        assert_point_cell(&game, POINT_1_1, Cell::Mine);
        assert_point_cell(&game, POINT_0_2, Cell::Empty(1));

        game.plant(POINT_1_2);

        num_mines(&game, 3);
        assert_eq!(game.available.len(), 9 * 9 - 3);
        assert_point_cell(&game, POINT_0_0, Cell::Mine);
        assert_point_cell(&game, POINT_0_1, Cell::Empty(3));
        assert_point_cell(&game, POINT_1_0, Cell::Empty(2));
        assert_point_cell(&game, POINT_1_1, Cell::Mine);
        assert_point_cell(&game, POINT_0_2, Cell::Empty(2));
    }

    #[test]
    fn unplant_mine_works() {
        let mut game = set_up_game();

        game.unplant(POINT_0_0, true);

        num_mines(&game, 4);
        assert_eq!(game.available.len(), 9 * 9 - 6);
        assert_point_cell(&game, POINT_0_0, Cell::Empty(0));
        assert_point_cell(&game, POINT_1_1, Cell::Empty(1));
    }

    #[test]
    fn unplant_cell_works() {
        let mut game = set_up_game();

        game.unplant(POINT_0_2, true);

        num_mines(&game, 4);
        assert_eq!(game.available.len(), 9 * 9 - 6);
        assert_point_cell(&game, POINT_0_2, Cell::Empty(0));
        assert_point_cell(&game, POINT_0_0, Cell::Mine);
        assert_point_cell(&game, POINT_1_1, Cell::Empty(1));
    }

    #[test]
    fn first_play_mine_works() {
        let mut game = set_up_game();

        let res = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: BoardPoint { row: 0, col: 0 },
            })
            .unwrap();
        assert_eq!(res.len(), 4);

        num_mines(&game, 4);
        assert_eq!(game.available.len(), 9 * 9 - 8);
        assert_point_cell(&game, POINT_0_0, Cell::Empty(0));
        point_cell_state(&game, POINT_0_0, true, Some(0));
        assert_point_cell(&game, POINT_1_1, Cell::Empty(2));
        point_cell_state(&game, POINT_1_1, true, Some(0));
        assert_point_cell(&game, POINT_1_2, Cell::Mine);
        point_cell_state(&game, POINT_1_2, false, None);
    }

    #[test]
    fn first_play_cell_works() {
        let mut game = set_up_game();

        let res = game.play(Play {
            player: 0,
            action: Action::Reveal,
            point: BoardPoint { col: 7, row: 7 },
        });
        assert_eq!(res.unwrap().len(), 9 * 9 - 8);

        num_mines(&game, 4);
        assert_eq!(game.available.len(), 4); // not mine and not revealed
        assert_point_cell(&game, BoardPoint { row: 8, col: 8 }, Cell::Empty(0));
        point_cell_state(&game, BoardPoint { row: 8, col: 8 }, true, Some(0));
        assert_point_cell(&game, POINT_1_1, Cell::Mine);
        point_cell_state(&game, POINT_1_1, false, None);
        assert_point_cell(&game, POINT_1_2, Cell::Mine);
        point_cell_state(&game, POINT_1_2, false, None);
    }

    #[test]
    fn second_click_mine_failure() {
        let mut game = set_up_game();

        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: POINT_0_0,
            })
            .unwrap();

        let cell_point = BoardPoint { row: 1, col: 2 };
        let res = game.play(Play {
            player: 0,
            action: Action::Reveal,
            point: cell_point,
        });
        assert!(matches!(res.unwrap(), PlayOutcome::Failure(_)));
    }

    #[test]
    fn second_click_cell_success() {
        let mut game = set_up_game();

        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: POINT_0_0,
            })
            .unwrap();

        let cell_point = BoardPoint { row: 0, col: 2 };
        game.unplant(cell_point, false); // guarantee not mine
        let res = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: cell_point,
            })
            .unwrap();
        assert!(matches!(&res, PlayOutcome::Success(_)));
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn flag_works() {
        let mut game = set_up_game();

        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: POINT_0_0,
            })
            .unwrap();

        let cell_point = BoardPoint { row: 1, col: 2 };
        let res = game
            .play(Play {
                player: 0,
                action: Action::Flag,
                point: cell_point,
            })
            .unwrap();
        assert!(matches!(res, PlayOutcome::Flag(_)));

        let res = game.play(Play {
            player: 0,
            action: Action::Reveal,
            point: cell_point,
        });
        assert!(res.is_err());
    }

    #[test]
    fn unplant_updated_works() {
        let mut game = set_up_game();

        let _ = game.play(Play {
            player: 0,
            action: Action::Reveal,
            point: BoardPoint { col: 7, row: 7 },
        });

        assert_point_cell(&game, POINT_2_2, Cell::Empty(3));
        assert_point_cell(&game, POINT_0_3, Cell::Empty(1));

        let updated = game.unplant(POINT_1_1, true);

        assert_point_cell(&game, POINT_2_2, Cell::Empty(0));
        assert_point_cell(&game, POINT_0_3, Cell::Empty(0));

        assert!(updated.contains(&POINT_2_2));
        assert!(updated.contains(&POINT_0_3));
        assert_eq!(updated.len(), 7);
    }

    #[test]
    fn unflag_works() {
        let mut game = set_up_game();

        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: POINT_0_0,
            })
            .unwrap();

        let cell_point = BoardPoint { row: 1, col: 2 };
        let _ = game
            .play(Play {
                player: 0,
                action: Action::Flag,
                point: cell_point,
            })
            .unwrap();
        let res = game
            .play(Play {
                player: 0,
                action: Action::Flag,
                point: cell_point,
            })
            .unwrap();
        assert!(matches!(res, PlayOutcome::Flag(_)));

        let res = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: cell_point,
            })
            .unwrap();
        assert!(matches!(res, PlayOutcome::Failure(_)));
    }

    #[test]
    fn double_click_works() {
        let mut game = set_up_game_no_superclick();

        let res = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: BoardPoint { row: 2, col: 2 },
            })
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_point_cell(&game, BoardPoint { row: 2, col: 2 }, Cell::Empty(3));

        let _ = game
            .play(Play {
                player: 0,
                action: Action::Flag,
                point: POINT_1_1,
            })
            .unwrap();
        let _ = game
            .play(Play {
                player: 0,
                action: Action::Flag,
                point: POINT_1_2,
            })
            .unwrap();
        let _ = game
            .play(Play {
                player: 0,
                action: Action::Flag,
                point: POINT_2_1,
            })
            .unwrap();

        let res = game
            .play(Play {
                player: 0,
                action: Action::RevealAdjacent,
                point: BoardPoint { row: 2, col: 2 },
            })
            .expect("double-click should work");
        assert!(res.len() == 9 * 9 - 9); // 5 is worst case scenario for replant
    }

    #[test]
    fn bad_double_click_fails() {
        let mut game = set_up_game_no_superclick();

        let res = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: BoardPoint { row: 2, col: 2 },
            })
            .unwrap();
        assert_eq!(res.len(), 1);
        assert_point_cell(&game, BoardPoint { row: 2, col: 2 }, Cell::Empty(3));

        let _ = game
            .play(Play {
                player: 0,
                action: Action::Flag,
                point: POINT_1_1,
            })
            .unwrap();
        let _ = game
            .play(Play {
                player: 0,
                action: Action::Flag,
                point: POINT_1_2,
            })
            .unwrap();

        let res = game.play(Play {
            player: 0,
            action: Action::RevealAdjacent,
            point: BoardPoint { row: 2, col: 2 },
        });
        assert!(res.is_err());
    }

    #[test]
    fn score_works() {
        let mut game = set_up_game();

        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: POINT_0_0,
            })
            .unwrap();

        num_mines(&game, 4);

        let cell_point = BoardPoint { row: 0, col: 2 };
        let res = game.play(Play {
            player: 0,
            action: Action::Reveal,
            point: cell_point,
        });
        assert!(matches!(res.unwrap(), PlayOutcome::Success(_)));
        assert_eq!(game.players[0].score, 5);
    }

    #[test]
    fn dead_errors() {
        let mut game = set_up_game();

        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: POINT_0_0,
            })
            .unwrap();
        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: BoardPoint { row: 1, col: 2 },
            })
            .unwrap();

        let res = game.play(Play {
            player: 0,
            action: Action::Reveal,
            point: BoardPoint { row: 3, col: 3 },
        });
        assert!(matches!(res, Err(..)));
    }

    #[test]
    fn revealed_errors() {
        let mut game = set_up_game();

        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: POINT_0_0,
            })
            .unwrap();

        let res = game.play(Play {
            player: 0,
            action: Action::Reveal,
            point: BoardPoint { row: 1, col: 1 },
        });
        assert!(matches!(res, Err(..)));
    }

    #[test]
    fn oob_errors() {
        let mut game = empty_game(2);

        let res = game.play(Play {
            player: 0,
            action: Action::Reveal,
            point: BoardPoint { col: 10, row: 0 },
        });
        assert!(matches!(res, Err(..)));

        let res = game.play(Play {
            player: 0,
            action: Action::Reveal,
            point: BoardPoint { col: 0, row: 10 },
        });
        assert!(matches!(res, Err(..)));
    }

    #[test]
    fn victory_works() {
        let mut game = set_up_game_no_superclick();

        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: POINT_0_1,
            })
            .unwrap();
        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: POINT_1_0,
            })
            .unwrap();
        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: BoardPoint { row: 8, col: 8 },
            })
            .unwrap();

        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: BoardPoint { row: 0, col: 2 },
            })
            .unwrap();
        let res = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: BoardPoint { row: 2, col: 0 },
            })
            .unwrap();
        assert!(matches!(res, PlayOutcome::Victory(..)));
        assert_eq!(game.players[0].score, 77);
    }

    #[test]
    fn replant_works() {
        let mut game = set_up_game();
        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: POINT_0_0,
            })
            .unwrap();
        num_mines(&game, 4);
        assert_ne!(game.board[POINT_1_1].0, Cell::Mine);
        assert_ne!(game.board[POINT_0_1].0, Cell::Mine);
        assert_ne!(game.board[POINT_1_0].0, Cell::Mine);
        assert_eq!(game.board[POINT_1_2].0, Cell::Mine);
        assert_eq!(game.board[POINT_2_1].0, Cell::Mine);

        let mut game = set_up_game();
        let _ = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: POINT_2_2,
            })
            .unwrap();
        num_mines(&game, 4);
        assert_ne!(game.board[POINT_1_1].0, Cell::Mine);
        assert_ne!(game.board[POINT_2_1].0, Cell::Mine);
        assert_ne!(game.board[POINT_1_2].0, Cell::Mine);
        assert_ne!(game.board[POINT_3_2].0, Cell::Mine);
        assert_ne!(game.board[POINT_3_3].0, Cell::Mine);
        assert_ne!(game.board[POINT_2_3].0, Cell::Mine);
        assert_eq!(game.board[POINT_0_0].0, Cell::Mine);
    }
}
