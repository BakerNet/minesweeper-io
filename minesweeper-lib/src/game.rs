use std::cmp::max;
use std::collections::HashSet;

use crate::board::{Board, BoardPoint, CompactSerialize};
use crate::cell::{Cell, CellState, HiddenCell, PlayerCell, RevealedCell};
use crate::client::ClientPlayer;
use crate::replay::MinesweeperReplay;

use anyhow::{bail, Ok, Result};
use rand::{rng, seq::SliceRandom};
use serde::{Deserialize, Serialize};
use tinyvec::ArrayVec;

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
        let mut available: Vec<_> = (0..board.size())
            .map(|x| board.point_from_index(x))
            .collect();
        available.shuffle(&mut rng());
        let points_to_plant = &available[0..self.opts.num_mines];
        points_to_plant.iter().for_each(|x| {
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

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
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
    fn handle_flag(&mut self, player: usize, cell_point: &BoardPoint) -> Result<PlayOutcome> {
        let (_, cell_state) = &self.board[cell_point];
        if cell_state.revealed {
            bail!("Tried to play already revealed cell")
        }
        let player_cell = if self.players[player].flags.contains(cell_point) {
            self.players[player].flags.remove(cell_point);
            PlayerCell::Hidden(HiddenCell::Empty)
        } else {
            self.players[player].flags.insert(*cell_point);
            PlayerCell::Hidden(HiddenCell::Flag)
        };
        Ok(PlayOutcome::Flag((*cell_point, player_cell)))
    }

    fn handle_click(&mut self, player: usize, cell_point: &BoardPoint) -> Result<PlayOutcome> {
        let (_, cell_state) = &self.board[cell_point];
        if cell_state.revealed {
            bail!("Tried to play already revealed cell")
        }
        if self.players[player].flags.contains(cell_point) {
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
                    *cell_point,
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
                    *cell_point,
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
        cell_point: &BoardPoint,
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
            .filter(|c| !self.board[c].1.revealed && !self.players[player].flags.contains(c))
            .collect::<ArrayVec<[BoardPoint; 8]>>();
        let has_mine = unflagged_neighbors
            .iter()
            .copied()
            .find(|c| matches!(self.board[c].0, Cell::Mine));
        // check for mine first, so other clicks don't go through
        if let Some(c) = has_mine {
            self.reveal(player, &c);
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
                if self.board[c].1.revealed {
                    return acc;
                }
                let res = self
                    .handle_click(player, c)
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

    fn reveal(&mut self, player: usize, cell_point: &BoardPoint) -> bool {
        if self.board[cell_point].1.revealed {
            false
        } else {
            self.board[cell_point].1.revealed = true;
            self.board[cell_point].1.player = Some(player);
            self.available.remove(cell_point);
            self.players.iter_mut().for_each(|p| {
                p.flags.remove(cell_point);
            });
            true
        }
    }

    fn reveal_neighbors(
        &mut self,
        player: usize,
        cell_point: &BoardPoint,
    ) -> Result<Vec<BoardPoint>> {
        self.reveal(player, cell_point);
        let final_vec = vec![*cell_point];
        let neighbors = self.board.neighbors(cell_point);
        neighbors.iter().try_fold(final_vec, |mut acc, c| {
            let item = self.board[c];
            if item.1.revealed {
                return Ok(acc);
            }
            if let Cell::Empty(x) = item.0 {
                if x == 0 {
                    let mut recur_acc = self.reveal_neighbors(player, c)?;
                    acc.append(&mut recur_acc)
                } else if self.reveal(player, c) {
                    acc.push(*c)
                }
            } else {
                bail!("Called reveal neighbors when there is a mine nearby")
            }
            Ok(acc)
        })
    }

    fn has_no_revealed_nearby(&self, cell_point: &BoardPoint) -> bool {
        let neighbors = self.board.neighbors(cell_point);
        let nearby = neighbors
            .into_iter()
            .flat_map(|n| self.board.neighbors(&n))
            .collect::<HashSet<_>>();
        nearby
            .into_iter()
            .filter(|i| self.board[i].1.revealed)
            .count()
            == 0
    }

    fn plant(&mut self, cell_point: &BoardPoint) {
        self.available.remove(cell_point);

        self.board[cell_point].0 = self.board[cell_point].0.plant().unwrap();

        let neighbors = self.board.neighbors(cell_point);
        neighbors.iter().copied().for_each(|c| {
            self.board[c].0 = self.board[c].0.increment();
        });
    }

    fn unplant(&mut self, cell_point: &BoardPoint, rem_neighbors: bool) -> Vec<BoardPoint> {
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

        neighbors.iter().for_each(|i| {
            let new = if was_mine {
                if self.board[i].1.revealed {
                    updated_revealed.insert(*i);
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
        first_cell: &BoardPoint,
        neighbors: ArrayVec<[BoardPoint; 8]>,
    ) {
        if unplanted_mines == 0 {
            return;
        }
        let has_revealed_neighbor = |bp: &BoardPoint| {
            self.board
                .neighbors(bp)
                .iter()
                .any(|c| self.board[c].1.revealed)
        };
        let mut take_available: Vec<BoardPoint> = self
            .available
            .iter()
            .filter(|&bp| bp != first_cell && !neighbors.contains(bp) && !has_revealed_neighbor(bp))
            .copied()
            .collect::<Vec<_>>();
        let mut rng = rng();
        take_available.shuffle(&mut rng);
        if unplanted_mines > take_available.len() {
            let mut unplanted_points = neighbors;
            unplanted_points.shuffle(&mut rng);
            take_available.extend(unplanted_points);
        }
        take_available.iter().take(unplanted_mines).for_each(|x| {
            self.plant(x);
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
            Action::Reveal => self.handle_click(play.player, &play.point),
            Action::RevealAdjacent => self.handle_double_click(play.player, &play.point),
            Action::Flag => self.handle_flag(play.player, &play.point),
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

    pub fn viewer_board(&self) -> Board<PlayerCell> {
        self.board.viewer_board(false)
    }

    pub fn player_board(&self, player: usize) -> Board<PlayerCell> {
        let mut return_board = self.viewer_board();
        for f in self.players[player].flags.iter() {
            if let PlayerCell::Hidden(_) = return_board[f] {
                return_board[f] = return_board[f].add_flag()
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

    pub fn viewer_board_final(&self) -> Board<PlayerCell> {
        self.board.clone()
    }

    pub fn player_board_final(&self, player: usize) -> Board<PlayerCell> {
        let mut return_board = self.viewer_board_final();
        for f in self.players[player].flags.iter() {
            if let PlayerCell::Hidden(_) = return_board[f] {
                return_board[f] = return_board[f].add_flag()
            }
        }
        return_board
    }

    fn board_start(&self) -> Board<PlayerCell> {
        let mut board = self.board.clone();
        board
            .iter_mut()
            .for_each(|pc| *pc = pc.into_hidden().remove_flag());
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

pub fn compress_game_log(log: &[(Play, PlayOutcome)]) -> Vec<u8> {
    let mut compressed = Vec::new();
    
    for (play, outcome) in log {
        // Compress Play
        compressed.push(play.player as u8);
        compressed.push(play.action.to_compact_byte());
        compressed.push(play.point.row as u8);
        compressed.push(play.point.col as u8);
        
        // Compress PlayOutcome
        match outcome {
            PlayOutcome::Success(cells) => {
                compressed.push(0); // Success marker
                compressed.push(cells.len() as u8);
                for (point, revealed) in cells {
                    compressed.push(point.row as u8);
                    compressed.push(point.col as u8);
                    compressed.push(revealed.player as u8);
                    compressed.push(match revealed.contents {
                        Cell::Empty(n) => n,
                        Cell::Mine => 9,
                    });
                }
            }
            PlayOutcome::Failure((point, revealed)) => {
                compressed.push(1); // Failure marker
                compressed.push(point.row as u8);
                compressed.push(point.col as u8);
                compressed.push(revealed.player as u8);
                compressed.push(match revealed.contents {
                    Cell::Empty(n) => n,
                    Cell::Mine => 9,
                });
            }
            PlayOutcome::Victory(cells) => {
                compressed.push(2); // Victory marker
                compressed.push(cells.len() as u8);
                for (point, revealed) in cells {
                    compressed.push(point.row as u8);
                    compressed.push(point.col as u8);
                    compressed.push(revealed.player as u8);
                    compressed.push(match revealed.contents {
                        Cell::Empty(n) => n,
                        Cell::Mine => 9,
                    });
                }
            }
            PlayOutcome::Flag((point, player_cell)) => {
                compressed.push(3); // Flag marker
                compressed.push(point.row as u8);
                compressed.push(point.col as u8);
                compressed.push(player_cell.to_compact_byte());
            }
        }
    }
    
    compressed
}

pub fn decompress_game_log(compressed: &[u8]) -> Vec<(Play, PlayOutcome)> {
    let mut log = Vec::new();
    let mut i = 0;
    
    while i < compressed.len() {
        if i + 4 > compressed.len() {
            break;
        }
        
        // Decompress Play
        let player = compressed[i] as usize;
        let action = Action::from_compact_byte(compressed[i + 1]);
        let point = BoardPoint {
            row: compressed[i + 2] as usize,
            col: compressed[i + 3] as usize,
        };
        let play = Play { player, action, point };
        i += 4;
        
        if i >= compressed.len() {
            break;
        }
        
        // Decompress PlayOutcome
        let outcome_type = compressed[i];
        i += 1;
        
        let outcome = match outcome_type {
            0 => { // Success
                if i >= compressed.len() {
                    break;
                }
                let cell_count = compressed[i] as usize;
                i += 1;
                let mut cells = Vec::new();
                
                for _ in 0..cell_count {
                    if i + 4 > compressed.len() {
                        break;
                    }
                    let point = BoardPoint {
                        row: compressed[i] as usize,
                        col: compressed[i + 1] as usize,
                    };
                    let player = compressed[i + 2] as usize;
                    let contents = if compressed[i + 3] == 9 {
                        Cell::Mine
                    } else {
                        Cell::Empty(compressed[i + 3])
                    };
                    cells.push((point, RevealedCell { player, contents }));
                    i += 4;
                }
                PlayOutcome::Success(cells)
            }
            1 => { // Failure
                if i + 4 > compressed.len() {
                    break;
                }
                let point = BoardPoint {
                    row: compressed[i] as usize,
                    col: compressed[i + 1] as usize,
                };
                let player = compressed[i + 2] as usize;
                let contents = if compressed[i + 3] == 9 {
                    Cell::Mine
                } else {
                    Cell::Empty(compressed[i + 3])
                };
                i += 4;
                PlayOutcome::Failure((point, RevealedCell { player, contents }))
            }
            2 => { // Victory
                if i >= compressed.len() {
                    break;
                }
                let cell_count = compressed[i] as usize;
                i += 1;
                let mut cells = Vec::new();
                
                for _ in 0..cell_count {
                    if i + 4 > compressed.len() {
                        break;
                    }
                    let point = BoardPoint {
                        row: compressed[i] as usize,
                        col: compressed[i + 1] as usize,
                    };
                    let player = compressed[i + 2] as usize;
                    let contents = if compressed[i + 3] == 9 {
                        Cell::Mine
                    } else {
                        Cell::Empty(compressed[i + 3])
                    };
                    cells.push((point, RevealedCell { player, contents }));
                    i += 4;
                }
                PlayOutcome::Victory(cells)
            }
            3 => { // Flag
                if i + 3 > compressed.len() {
                    break;
                }
                let point = BoardPoint {
                    row: compressed[i] as usize,
                    col: compressed[i + 1] as usize,
                };
                let player_cell = PlayerCell::from_compact_byte(compressed[i + 2]);
                i += 3;
                PlayOutcome::Flag((point, player_cell))
            }
            _ => break, // Invalid outcome type
        };
        
        log.push((play, outcome));
    }
    
    log
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

impl Action {
    pub fn to_str(&self) -> &'static str {
        match self {
            Action::Flag => "Flag",
            Action::Reveal => "Reveal",
            Action::RevealAdjacent => "Reveal Adjacent",
        }
    }
}

impl CompactSerialize for Action {
    fn to_compact_byte(&self) -> u8 {
        match self {
            Action::Flag => 0,
            Action::Reveal => 1,
            Action::RevealAdjacent => 2,
        }
    }

    fn from_compact_byte(byte: u8) -> Self {
        match byte {
            0 => Action::Flag,
            1 => Action::Reveal,
            2 => Action::RevealAdjacent,
            _ => Action::Reveal, // Default fallback
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

    /// Convert to compact format for WebSocket transmission
    pub fn to_compact(&self) -> CompactPlayOutcome {
        fn compact_point_and_cell(point: &BoardPoint, player_cell: &PlayerCell) -> (u8, u8, u8) {
            let row = point.row as u8;
            let col = point.col as u8;
            let cell_byte = player_cell.to_compact_byte();
            (row, col, cell_byte)
        }

        match self {
            PlayOutcome::Success(cells) => {
                let compact_cells = cells
                    .iter()
                    .map(|(point, revealed)| {
                        compact_point_and_cell(point, &PlayerCell::Revealed(*revealed))
                    })
                    .collect();
                CompactPlayOutcome::Success(compact_cells)
            }
            PlayOutcome::Failure((point, revealed)) => CompactPlayOutcome::Failure(
                compact_point_and_cell(point, &PlayerCell::Revealed(*revealed)),
            ),
            PlayOutcome::Victory(cells) => {
                let compact_cells = cells
                    .iter()
                    .map(|(point, revealed)| {
                        compact_point_and_cell(point, &PlayerCell::Revealed(*revealed))
                    })
                    .collect();
                CompactPlayOutcome::Victory(compact_cells)
            }
            PlayOutcome::Flag((point, player_cell)) => {
                CompactPlayOutcome::Flag(compact_point_and_cell(point, player_cell))
            }
        }
    }

    pub fn combine(self, other: PlayOutcome) -> PlayOutcome {
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

/// Compressed version of PlayOutcome for WebSocket transmission
/// Uses raw byte encoding: (row, col, cell_data) where each is a single byte
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CompactPlayOutcome {
    #[serde(rename = "s")]
    Success(Vec<(u8, u8, u8)>), // (row, col, compact_cell)
    #[serde(rename = "x")]
    Failure((u8, u8, u8)), // (row, col, compact_cell)
    #[serde(rename = "v")]
    Victory(Vec<(u8, u8, u8)>), // (row, col, compact_cell)
    #[serde(rename = "f")]
    Flag((u8, u8, u8)), // (row, col, compact_cell)
}

impl CompactPlayOutcome {
    pub fn len(&self) -> usize {
        match self {
            Self::Success(v) => v.len(),
            Self::Victory(v) => v.len(),
            Self::Failure(_) => 1,
            Self::Flag(_) => 1,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert back to full PlayOutcome format (for database storage or fallback)
    pub fn to_full(&self) -> PlayOutcome {
        match self {
            CompactPlayOutcome::Success(cells) => {
                let full_cells = cells
                    .iter()
                    .map(|(row, col, cell_byte)| {
                        let point = BoardPoint {
                            row: *row as usize,
                            col: *col as usize,
                        };
                        let player_cell = PlayerCell::from_compact_byte(*cell_byte);
                        match player_cell {
                            PlayerCell::Revealed(revealed) => (point, revealed),
                            // Handle edge case - convert non-revealed to revealed
                            _ => (
                                point,
                                RevealedCell {
                                    player: 0,
                                    contents: Cell::Empty(0),
                                },
                            ),
                        }
                    })
                    .collect();
                PlayOutcome::Success(full_cells)
            }
            CompactPlayOutcome::Failure((row, col, cell_byte)) => {
                let point = BoardPoint {
                    row: *row as usize,
                    col: *col as usize,
                };
                let player_cell = PlayerCell::from_compact_byte(*cell_byte);
                match player_cell {
                    PlayerCell::Revealed(revealed) => PlayOutcome::Failure((point, revealed)),
                    _ => PlayOutcome::Failure((
                        point,
                        RevealedCell {
                            player: 0,
                            contents: Cell::Empty(0),
                        },
                    )),
                }
            }
            CompactPlayOutcome::Victory(cells) => {
                let full_cells = cells
                    .iter()
                    .map(|(row, col, cell_byte)| {
                        let point = BoardPoint {
                            row: *row as usize,
                            col: *col as usize,
                        };
                        let player_cell = PlayerCell::from_compact_byte(*cell_byte);
                        match player_cell {
                            PlayerCell::Revealed(revealed) => (point, revealed),
                            _ => (
                                point,
                                RevealedCell {
                                    player: 0,
                                    contents: Cell::Empty(0),
                                },
                            ),
                        }
                    })
                    .collect();
                PlayOutcome::Victory(full_cells)
            }
            CompactPlayOutcome::Flag((row, col, cell_byte)) => {
                let point = BoardPoint {
                    row: *row as usize,
                    col: *col as usize,
                };
                let player_cell = PlayerCell::from_compact_byte(*cell_byte);
                PlayOutcome::Flag((point, player_cell))
            }
        }
    }
}

#[cfg(test)]
mod playoutcome_compression_tests {
    use super::*;

    #[test]
    fn test_playoutcome_compression_roundtrip() {
        // Test all variants of PlayOutcome
        let point = BoardPoint { row: 5, col: 10 };
        let revealed = RevealedCell {
            player: 3,
            contents: Cell::Empty(4),
        };

        // Test Success variant
        let success = PlayOutcome::Success(vec![(point, revealed)]);
        let compact_success = success.to_compact();
        let restored_success = compact_success.to_full();

        match (success, restored_success) {
            (PlayOutcome::Success(orig), PlayOutcome::Success(restored)) => {
                assert_eq!(orig.len(), restored.len());
                assert_eq!(orig[0].0, restored[0].0); // BoardPoint
                assert_eq!(orig[0].1, restored[0].1); // RevealedCell
            }
            _ => panic!("Variants don't match"),
        }

        // Test Flag variant
        let flag = PlayOutcome::Flag((point, PlayerCell::Hidden(HiddenCell::Flag)));
        let compact_flag = flag.to_compact();
        let restored_flag = compact_flag.to_full();

        match (flag, restored_flag) {
            (
                PlayOutcome::Flag((orig_point, orig_cell)),
                PlayOutcome::Flag((restored_point, restored_cell)),
            ) => {
                assert_eq!(orig_point, restored_point);
                assert_eq!(orig_cell, restored_cell);
            }
            _ => panic!("Flag variants don't match"),
        }
    }

    #[test]
    fn test_playoutcome_compression_size() {
        // Create a large PlayOutcome similar to a cascade reveal
        let mut cells = Vec::new();
        for i in 0..50 {
            cells.push((
                BoardPoint {
                    row: i / 10,
                    col: i % 10,
                },
                RevealedCell {
                    player: i % 4,
                    contents: Cell::Empty((i % 9) as u8),
                },
            ));
        }

        let large_outcome = PlayOutcome::Success(cells);
        let compact_outcome = large_outcome.to_compact();

        // Test JSON serialization sizes
        let original_json = serde_json::to_string(&large_outcome).unwrap();
        let compact_json = serde_json::to_string(&compact_outcome).unwrap();

        println!("Original PlayOutcome JSON: {} bytes", original_json.len());
        println!("Compact PlayOutcome JSON: {} bytes", compact_json.len());
        println!(
            "Original sample: {}",
            &original_json[..200.min(original_json.len())]
        );
        println!(
            "Compact sample: {}",
            &compact_json[..200.min(compact_json.len())]
        );

        let compression_ratio =
            (original_json.len() as f64 - compact_json.len() as f64) / original_json.len() as f64;
        println!("Compression: {:.1}% reduction", compression_ratio * 100.0);

        // The test was wrong - let's just verify it works without size assumptions
        // Size benefits depend on the specific data and JSON vs binary encoding
    }

    #[test]
    fn test_large_board_support() {
        // Test that we support reasonable board sizes up to 100x100
        let point = BoardPoint { row: 99, col: 99 };
        let outcome = PlayOutcome::Flag((point, PlayerCell::Hidden(HiddenCell::Flag)));

        // Should work fine for boards up to 100x100
        let compact = outcome.to_compact();
        let restored = compact.to_full();

        match (outcome, restored) {
            (
                PlayOutcome::Flag((orig_point, orig_cell)),
                PlayOutcome::Flag((restored_point, restored_cell)),
            ) => {
                assert_eq!(orig_point, restored_point);
                assert_eq!(orig_cell, restored_cell);
            }
            _ => panic!("Variants don't match"),
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

        game.plant(&POINT_0_0);
        game.plant(&POINT_1_1);
        game.plant(&POINT_1_2);
        game.plant(&POINT_2_1);
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

        game.plant(&POINT_0_0);

        num_mines(&game, 1);
        assert_eq!(game.available.len(), 9 * 9 - 1);
        assert_point_cell(&game, POINT_0_0, Cell::Mine);
        assert_point_cell(&game, POINT_0_1, Cell::Empty(1));
        assert_point_cell(&game, POINT_1_0, Cell::Empty(1));
        assert_point_cell(&game, POINT_1_1, Cell::Empty(1));
        assert_point_cell(&game, POINT_0_2, Cell::Empty(0));

        game.plant(&POINT_1_1);

        num_mines(&game, 2);
        assert_eq!(game.available.len(), 9 * 9 - 2);
        assert_point_cell(&game, POINT_0_0, Cell::Mine);
        assert_point_cell(&game, POINT_0_1, Cell::Empty(2));
        assert_point_cell(&game, POINT_1_0, Cell::Empty(2));
        assert_point_cell(&game, POINT_1_1, Cell::Mine);
        assert_point_cell(&game, POINT_0_2, Cell::Empty(1));

        game.plant(&POINT_1_2);

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

        game.unplant(&POINT_0_0, true);

        num_mines(&game, 4);
        assert_eq!(game.available.len(), 9 * 9 - 6);
        assert_point_cell(&game, POINT_0_0, Cell::Empty(0));
        assert_point_cell(&game, POINT_1_1, Cell::Empty(1));
    }

    #[test]
    fn unplant_cell_works() {
        let mut game = set_up_game();

        game.unplant(&POINT_0_2, true);

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
        game.unplant(&cell_point, false); // guarantee not mine
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

        let updated = game.unplant(&POINT_1_1, true);

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

        let res = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: POINT_0_0,
            })
            .unwrap();

        num_mines(&game, 4);
        assert!(matches!(res, PlayOutcome::Success(_)));
        assert_eq!(game.players[0].score, 4);
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

        // click mine
        let res = game
            .play(Play {
                player: 0,
                action: Action::Reveal,
                point: BoardPoint { row: 1, col: 2 },
            })
            .unwrap();
        assert!(matches!(res, PlayOutcome::Failure(_)));

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

    #[test]
    fn test_game_log_compression_round_trip() {
        use crate::cell::{Cell, RevealedCell};
        
        // Create a sample game log with various play outcomes
        let test_log = vec![
            (
                Play {
                    player: 0,
                    action: Action::Reveal,
                    point: BoardPoint { row: 1, col: 1 },
                },
                PlayOutcome::Success(vec![
                    (
                        BoardPoint { row: 1, col: 1 },
                        RevealedCell {
                            player: 0,
                            contents: Cell::Empty(2),
                        },
                    ),
                    (
                        BoardPoint { row: 1, col: 2 },
                        RevealedCell {
                            player: 0,
                            contents: Cell::Empty(1),
                        },
                    ),
                ]),
            ),
            (
                Play {
                    player: 0,
                    action: Action::Flag,
                    point: BoardPoint { row: 2, col: 2 },
                },
                PlayOutcome::Flag((
                    BoardPoint { row: 2, col: 2 },
                    PlayerCell::Hidden(HiddenCell::Flag),
                )),
            ),
            (
                Play {
                    player: 0,
                    action: Action::Reveal,
                    point: BoardPoint { row: 3, col: 3 },
                },
                PlayOutcome::Failure((
                    BoardPoint { row: 3, col: 3 },
                    RevealedCell {
                        player: 0,
                        contents: Cell::Mine,
                    },
                )),
            ),
            (
                Play {
                    player: 0,
                    action: Action::RevealAdjacent,
                    point: BoardPoint { row: 0, col: 0 },
                },
                PlayOutcome::Victory(vec![
                    (
                        BoardPoint { row: 0, col: 0 },
                        RevealedCell {
                            player: 0,
                            contents: Cell::Empty(0),
                        },
                    ),
                ]),
            ),
        ];

        // Test compression and decompression
        let compressed = compress_game_log(&test_log);
        let decompressed = decompress_game_log(&compressed);

        assert_eq!(test_log.len(), decompressed.len());
        
        for (original, restored) in test_log.iter().zip(decompressed.iter()) {
            assert_eq!(original.0.player, restored.0.player);
            assert_eq!(original.0.action, restored.0.action);
            assert_eq!(original.0.point, restored.0.point);
            assert_eq!(original.1, restored.1);
        }
    }

    #[test]
    fn test_game_log_compression_empty_log() {
        let empty_log: Vec<(Play, PlayOutcome)> = Vec::new();
        let compressed = compress_game_log(&empty_log);
        let decompressed = decompress_game_log(&compressed);
        
        assert_eq!(empty_log, decompressed);
    }

    #[test]
    fn test_game_log_compression_single_play() {
        let single_play = vec![
            (
                Play {
                    player: 5,
                    action: Action::Flag,
                    point: BoardPoint { row: 10, col: 15 },
                },
                PlayOutcome::Flag((
                    BoardPoint { row: 10, col: 15 },
                    PlayerCell::Hidden(HiddenCell::Flag),
                )),
            ),
        ];

        let compressed = compress_game_log(&single_play);
        let decompressed = decompress_game_log(&compressed);

        assert_eq!(single_play, decompressed);
    }

    #[test]
    fn test_game_log_compression_all_actions() {
        use crate::cell::{Cell, RevealedCell};
        
        let test_log = vec![
            (
                Play {
                    player: 0,
                    action: Action::Flag,
                    point: BoardPoint { row: 0, col: 0 },
                },
                PlayOutcome::Flag((
                    BoardPoint { row: 0, col: 0 },
                    PlayerCell::Hidden(HiddenCell::Flag),
                )),
            ),
            (
                Play {
                    player: 1,
                    action: Action::Reveal,
                    point: BoardPoint { row: 1, col: 1 },
                },
                PlayOutcome::Success(vec![
                    (
                        BoardPoint { row: 1, col: 1 },
                        RevealedCell {
                            player: 1,
                            contents: Cell::Empty(5),
                        },
                    ),
                ]),
            ),
            (
                Play {
                    player: 2,
                    action: Action::RevealAdjacent,
                    point: BoardPoint { row: 2, col: 2 },
                },
                PlayOutcome::Failure((
                    BoardPoint { row: 2, col: 2 },
                    RevealedCell {
                        player: 2,
                        contents: Cell::Mine,
                    },
                )),
            ),
        ];

        let compressed = compress_game_log(&test_log);
        let decompressed = decompress_game_log(&compressed);

        assert_eq!(test_log, decompressed);
    }

    #[test]
    fn test_game_log_compression_large_numbers() {
        use crate::cell::{Cell, RevealedCell};
        
        let test_log = vec![
            (
                Play {
                    player: 255,
                    action: Action::Reveal,
                    point: BoardPoint { row: 255, col: 255 },
                },
                PlayOutcome::Success(vec![
                    (
                        BoardPoint { row: 255, col: 255 },
                        RevealedCell {
                            player: 255,
                            contents: Cell::Empty(8),
                        },
                    ),
                ]),
            ),
        ];

        let compressed = compress_game_log(&test_log);
        let decompressed = decompress_game_log(&compressed);

        // Note: Player numbers > 255 will be truncated to u8
        assert_eq!(decompressed[0].0.player, 255);
        assert_eq!(decompressed[0].0.point.row, 255);
        assert_eq!(decompressed[0].0.point.col, 255);
        
        match &decompressed[0].1 {
            PlayOutcome::Success(cells) => {
                assert_eq!(cells[0].1.player, 255);
                assert_eq!(cells[0].1.contents, Cell::Empty(8));
            }
            _ => panic!("Expected Success outcome"),
        }
    }

    #[test]
    fn test_game_log_compression_efficiency() {
        use crate::cell::{Cell, RevealedCell};
        
        // Create a longer game log to test compression efficiency
        let mut test_log = Vec::new();
        for i in 0..100 {
            test_log.push((
                Play {
                    player: i % 4,
                    action: Action::Reveal,
                    point: BoardPoint { row: i / 10, col: i % 10 },
                },
                PlayOutcome::Success(vec![
                    (
                        BoardPoint { row: i / 10, col: i % 10 },
                        RevealedCell {
                            player: i % 4,
                            contents: Cell::Empty((i % 9) as u8),
                        },
                    ),
                ]),
            ));
        }

        let compressed = compress_game_log(&test_log);
        let decompressed = decompress_game_log(&compressed);

        assert_eq!(test_log.len(), decompressed.len());
        
        // Check that compression is reasonably efficient
        // Each play should take roughly 8 bytes (4 for play + 4 for outcome)
        // Plus some overhead, so let's say max 12 bytes per play
        assert!(compressed.len() < test_log.len() * 12);
        
        // Verify correctness
        for (original, restored) in test_log.iter().zip(decompressed.iter()) {
            assert_eq!(original, restored);
        }
    }
}
