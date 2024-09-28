use std::cmp::Ordering;

use anyhow::{bail, Result};

use crate::{
    analysis::AnalyzedCell,
    board::{Board, BoardPoint},
    cell::{HiddenCell, PlayerCell},
    client::ClientPlayer,
    game::{Play, PlayOutcome},
};

mod analysis;

pub use analysis::MinesweeperReplayAnalysis;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReplayAnalysisCell(pub PlayerCell, pub Option<AnalyzedCell>);

#[derive(Debug, PartialEq, Eq)]
pub enum ReplayPosition {
    End,
    Beginning,
    Other(usize),
}

impl ReplayPosition {
    pub fn from_pos(pos: usize, len: usize) -> Self {
        match pos {
            p if p == len - 1 => ReplayPosition::End,
            0 => ReplayPosition::Beginning,
            default => ReplayPosition::Other(default),
        }
    }

    pub fn to_num(&self, len: usize) -> usize {
        match self {
            ReplayPosition::End => len,
            ReplayPosition::Beginning => 0,
            ReplayPosition::Other(default) => *default,
        }
    }

    pub fn is_valid(&self, len: usize) -> bool {
        match self {
            ReplayPosition::End => true,
            ReplayPosition::Beginning => true,
            ReplayPosition::Other(x) => *x != 0 && *x < len,
        }
    }
}

impl PartialOrd for ReplayPosition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (ReplayPosition::End, ReplayPosition::End) => Some(Ordering::Equal),
            (ReplayPosition::End, _) => Some(Ordering::Greater),
            (ReplayPosition::Beginning, ReplayPosition::Beginning) => Some(Ordering::Equal),
            (ReplayPosition::Beginning, _) => Some(Ordering::Less),
            (ReplayPosition::Other(_), ReplayPosition::End) => Some(Ordering::Less),
            (ReplayPosition::Other(_), ReplayPosition::Beginning) => Some(Ordering::Greater),
            (ReplayPosition::Other(s), ReplayPosition::Other(o)) => Some(s.cmp(o)),
        }
    }
}

pub trait Replayable {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        false
    }
    fn current_pos(&self) -> ReplayPosition;
    fn advance(&mut self) -> Result<ReplayPosition>;
    fn rewind(&mut self) -> Result<ReplayPosition>;
    fn to_pos(&mut self, pos: ReplayPosition) -> Result<ReplayPosition> {
        let len = self.len();
        if !pos.is_valid(len) {
            bail!(
                "Called to_pos with pos out of bounds (max {}): {:?}",
                self.len() - 1,
                pos
            )
        }
        while pos < self.current_pos() {
            let _ = self.rewind();
        }
        while pos > self.current_pos() {
            let _ = self.advance();
        }
        let new_pos = self.current_pos();
        assert_eq!(pos, new_pos);
        Ok(new_pos)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SimplePlayer {
    score: usize,
    dead: bool,
    victory_click: bool,
}

impl SimplePlayer {
    pub fn update_client_player(self, cp: &mut ClientPlayer) {
        cp.top_score = false;
        cp.score = self.score;
        cp.dead = self.dead;
        cp.victory_click = self.victory_click;
    }
}

pub struct MinesweeperReplay {
    current_play: Option<Play>,
    current_board: Board<PlayerCell>,
    current_players: Vec<SimplePlayer>,
    current_flags: usize,
    current_revealed_mines: usize,
    log: Vec<(Play, PlayOutcome)>,
    current_pos: usize,
}

impl MinesweeperReplay {
    pub fn new(
        starting_board: Board<PlayerCell>,
        log: Vec<(Play, PlayOutcome)>,
        players: usize,
    ) -> Self {
        Self {
            current_board: starting_board,
            current_play: None,
            current_players: vec![SimplePlayer::default(); players],
            current_flags: 0,
            current_revealed_mines: 0,
            log,
            current_pos: 0,
        }
    }

    pub fn with_analysis(self) -> MinesweeperReplayWithAnalysis {
        let mut replay = self;
        let analysis = MinesweeperReplayAnalysis::from_replay(&mut replay);
        let mut current_board = Board::new(
            replay.current_board.rows(),
            replay.current_board.cols(),
            ReplayAnalysisCell(PlayerCell::Hidden(HiddenCell::Empty), None::<AnalyzedCell>),
        );
        replay
            .current_board
            .rows_iter()
            .enumerate()
            .for_each(|(row, v)| {
                v.iter().enumerate().for_each(|(col, c)| {
                    let point = BoardPoint { row, col };
                    let curr = current_board[point];
                    current_board[point] = ReplayAnalysisCell(*c, curr.1);
                })
            });
        MinesweeperReplayWithAnalysis {
            replay,
            analysis,
            current_board,
        }
    }

    pub fn current_play(&self) -> Option<Play> {
        self.current_play
    }

    pub fn current_board(&self) -> &Board<PlayerCell> {
        &self.current_board
    }

    pub fn current_players(&self) -> &Vec<SimplePlayer> {
        &self.current_players
    }

    pub fn current_flags_and_revealed_mines(&self) -> usize {
        self.current_flags + self.current_revealed_mines
    }
}

impl Replayable for MinesweeperReplay {
    fn len(&self) -> usize {
        self.log.len() + 1
    }

    fn current_pos(&self) -> ReplayPosition {
        ReplayPosition::from_pos(self.current_pos, self.len())
    }

    fn advance(&mut self) -> Result<ReplayPosition> {
        if self.current_pos == self.len() - 1 {
            bail!("Called next on end")
        }
        let play = &self.log[self.current_pos];
        self.current_play = Some(play.0);
        match &play.1 {
            PlayOutcome::Success(results) => results.iter().for_each(|rc| {
                self.current_players[rc.1.player].score += 1;
                self.current_board[rc.0] = PlayerCell::Revealed(rc.1);
            }),
            PlayOutcome::Failure(rc) => {
                self.current_players[rc.1.player].dead = true;
                self.current_revealed_mines += 1;
                self.current_board[rc.0] = PlayerCell::Revealed(rc.1);
            }
            PlayOutcome::Victory(results) => {
                self.current_players[results[0].1.player].victory_click = true;
                results.iter().for_each(|rc| {
                    self.current_players[rc.1.player].score += 1;
                    self.current_board[rc.0] = PlayerCell::Revealed(rc.1);
                });
            }
            PlayOutcome::Flag(res) => {
                if matches!(res.1, PlayerCell::Hidden(HiddenCell::Flag)) {
                    self.current_flags += 1;
                    self.current_board[res.0] = self.current_board[res.0].add_flag()
                } else {
                    self.current_flags -= 1;
                    self.current_board[res.0] = self.current_board[res.0].remove_flag()
                }
            }
        };
        self.current_pos += 1;
        Ok(self.current_pos())
    }

    fn rewind(&mut self) -> Result<ReplayPosition> {
        if self.current_pos == 0 {
            bail!("Called prev on start")
        }
        self.current_pos -= 1;
        let play_to_undo = &self.log[self.current_pos];
        self.current_play = if self.current_pos == 0 {
            None
        } else {
            Some(self.log[self.current_pos - 1].0)
        };
        match &play_to_undo.1 {
            PlayOutcome::Success(results) => results.iter().for_each(|rc| {
                self.current_players[rc.1.player].score -= 1;
                self.current_board[rc.0] = PlayerCell::Hidden(HiddenCell::Empty);
            }),
            PlayOutcome::Failure(rc) => {
                self.current_players[rc.1.player].dead = false;
                self.current_revealed_mines -= 1;
                self.current_board[rc.0] = PlayerCell::Hidden(HiddenCell::Mine);
            }
            PlayOutcome::Victory(results) => {
                self.current_players[results[0].1.player].victory_click = false;
                results.iter().for_each(|rc| {
                    self.current_players[rc.1.player].score -= 1;
                    self.current_board[rc.0] = PlayerCell::Hidden(HiddenCell::Empty);
                });
            }
            PlayOutcome::Flag(res) => {
                if matches!(res.1, PlayerCell::Hidden(HiddenCell::Flag)) {
                    self.current_flags -= 1;
                    self.current_board[res.0] = self.current_board[res.0].remove_flag()
                } else {
                    self.current_flags += 1;
                    self.current_board[res.0] = self.current_board[res.0].add_flag()
                }
            }
        };
        Ok(self.current_pos())
    }
}

pub struct MinesweeperReplayWithAnalysis {
    replay: MinesweeperReplay,
    analysis: MinesweeperReplayAnalysis,
    current_board: Board<ReplayAnalysisCell>,
}

impl MinesweeperReplayWithAnalysis {
    pub fn current_play(&self) -> Option<Play> {
        self.replay.current_play
    }

    pub fn current_board(&self) -> &Board<ReplayAnalysisCell> {
        &self.current_board
    }

    pub fn current_players(&self) -> &Vec<SimplePlayer> {
        &self.replay.current_players
    }

    pub fn current_flags_and_revealed_mines(&self) -> usize {
        self.replay.current_flags + self.replay.current_revealed_mines
    }

    fn update_current_board(&mut self) {
        let replay_board = self.replay.current_board();
        let analysis_board = self.analysis.current_board();
        replay_board
            .iter()
            .zip(analysis_board.iter())
            .enumerate()
            .for_each(|(i, (pc, ac))| {
                let point = self.current_board.point_from_index(i);
                self.current_board[point] = ReplayAnalysisCell(pc.to_owned(), ac.to_owned());
            });
    }
}

impl Replayable for MinesweeperReplayWithAnalysis {
    fn len(&self) -> usize {
        self.replay.len()
    }

    fn current_pos(&self) -> ReplayPosition {
        self.replay.current_pos()
    }

    fn advance(&mut self) -> Result<ReplayPosition> {
        let _ = self.analysis.advance();
        let ret = self.replay.advance();
        if ret.is_ok() {
            self.update_current_board()
        }
        ret
    }

    fn rewind(&mut self) -> Result<ReplayPosition> {
        let _ = self.analysis.rewind();
        let ret = self.replay.rewind();
        if ret.is_ok() {
            self.update_current_board()
        }
        ret
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        board::BoardPoint,
        cell::{Cell, RevealedCell},
        game::Action,
    };

    pub const MINES: [BoardPoint; 4] = [
        BoardPoint { row: 0, col: 3 },
        BoardPoint { row: 3, col: 0 },
        BoardPoint { row: 3, col: 2 },
        BoardPoint { row: 3, col: 3 },
    ];
    pub const PLAY_1_RES: [(BoardPoint, RevealedCell); 9] = [
        (
            BoardPoint { row: 0, col: 0 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(0),
            },
        ),
        (
            BoardPoint { row: 0, col: 1 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(0),
            },
        ),
        (
            BoardPoint { row: 0, col: 2 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(1),
            },
        ),
        (
            BoardPoint { row: 1, col: 0 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(0),
            },
        ),
        (
            BoardPoint { row: 1, col: 1 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(0),
            },
        ),
        (
            BoardPoint { row: 1, col: 2 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(1),
            },
        ),
        (
            BoardPoint { row: 2, col: 0 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(1),
            },
        ),
        (
            BoardPoint { row: 2, col: 1 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(2),
            },
        ),
        (
            BoardPoint { row: 2, col: 2 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(2),
            },
        ),
    ];
    pub const PLAY_2_RES: (BoardPoint, PlayerCell) = (
        BoardPoint { row: 3, col: 2 },
        PlayerCell::Hidden(HiddenCell::Flag),
    );
    pub const PLAY_3_RES: (BoardPoint, RevealedCell) = (
        BoardPoint { row: 2, col: 3 },
        RevealedCell {
            player: 0,
            contents: Cell::Empty(2),
        },
    );
    pub const PLAY_4_RES: (BoardPoint, RevealedCell) = (
        BoardPoint { row: 3, col: 3 },
        RevealedCell {
            player: 0,
            contents: Cell::Mine,
        },
    );

    #[test]
    fn test_replay() {
        let mut expected_starting_board = Board::new(4, 4, PlayerCell::Hidden(HiddenCell::Empty));
        MINES.iter().for_each(|point| {
            expected_starting_board[*point] = PlayerCell::Hidden(HiddenCell::Mine);
        });
        let expected_starting_board = expected_starting_board;

        let mut expected_next_board = expected_starting_board.clone();
        // res of first play
        PLAY_1_RES.iter().for_each(|(point, rc)| {
            expected_next_board[*point] = PlayerCell::Revealed(*rc);
        });
        let expected_board_1 = expected_next_board.clone();
        // res of second play
        expected_next_board[PLAY_2_RES.0] = PlayerCell::Hidden(HiddenCell::FlagMine);
        let expected_board_2 = expected_next_board.clone();
        // res of third play
        expected_next_board[PLAY_3_RES.0] = PlayerCell::Revealed(PLAY_3_RES.1);
        let expected_board_3 = expected_next_board.clone();
        // res of final play
        expected_next_board[PLAY_4_RES.0] = PlayerCell::Revealed(PLAY_4_RES.1);
        let expected_final_board = expected_next_board.clone();

        drop(expected_next_board);

        let mut replay = MinesweeperReplay::new(
            expected_starting_board.clone(),
            Vec::from([
                (
                    Play {
                        player: 0,
                        action: Action::Reveal,
                        point: BoardPoint { row: 2, col: 2 },
                    },
                    PlayOutcome::Success(Vec::from(PLAY_1_RES)),
                ),
                (
                    Play {
                        player: 0,
                        action: Action::Flag,
                        point: BoardPoint { row: 3, col: 2 },
                    },
                    PlayOutcome::Flag(PLAY_2_RES),
                ),
                (
                    Play {
                        player: 0,
                        action: Action::Reveal,
                        point: BoardPoint { row: 2, col: 3 },
                    },
                    PlayOutcome::Success(Vec::from([PLAY_3_RES])),
                ),
                (
                    Play {
                        player: 0,
                        action: Action::Reveal,
                        point: BoardPoint { row: 3, col: 3 },
                    },
                    PlayOutcome::Failure(PLAY_4_RES),
                ),
            ]),
            2,
        );

        // test defaults
        assert_eq!(replay.current_players.len(), 2);
        assert_eq!(
            replay
                .current_players
                .iter()
                .map(|p| p.score)
                .sum::<usize>(),
            0
        );
        assert_eq!(replay.current_flags, 0);
        assert_eq!(replay.current_revealed_mines, 0);
        assert_eq!(replay.len(), 5);

        // test advance
        assert!(matches!(replay.advance(), Ok(ReplayPosition::Other(1))));
        assert_eq!(replay.current_board(), &expected_board_1);
        assert!(matches!(replay.advance(), Ok(ReplayPosition::Other(2))));
        assert_eq!(replay.current_board(), &expected_board_2);
        assert!(matches!(replay.advance(), Ok(ReplayPosition::Other(3))));
        assert_eq!(replay.current_board(), &expected_board_3);
        assert!(matches!(replay.advance(), Ok(ReplayPosition::End)));
        assert_eq!(replay.current_board(), &expected_final_board);

        // should error on advance at end
        assert!(replay.advance().is_err());

        // test rewind
        assert!(matches!(replay.rewind(), Ok(ReplayPosition::Other(3))));
        assert_eq!(replay.current_board(), &expected_board_3);
        assert!(matches!(replay.rewind(), Ok(ReplayPosition::Other(2))));
        assert_eq!(replay.current_board(), &expected_board_2);
        assert!(matches!(replay.rewind(), Ok(ReplayPosition::Other(1))));
        assert_eq!(replay.current_board(), &expected_board_1);
        assert!(matches!(replay.rewind(), Ok(ReplayPosition::Beginning)));
        assert_eq!(replay.current_board(), &expected_starting_board);

        // should error on rewind at beginning
        assert!(replay.rewind().is_err());

        // try to_pos (auto advance/rewind)
        assert!(matches!(
            replay.to_pos(ReplayPosition::Other(2)),
            Ok(ReplayPosition::Other(2))
        ));
        assert_eq!(replay.current_board(), &expected_board_2);
        assert!(matches!(
            replay.to_pos(ReplayPosition::End),
            Ok(ReplayPosition::End)
        ));
        assert_eq!(replay.current_board(), &expected_final_board);
        assert!(matches!(
            replay.to_pos(ReplayPosition::Other(1)),
            Ok(ReplayPosition::Other(1))
        ));
        assert_eq!(replay.current_board(), &expected_board_1);

        assert!(replay.to_pos(ReplayPosition::Other(5)).is_err());

        // try to_pos (auto advance/rewind)
        assert!(matches!(
            replay.to_pos(ReplayPosition::Other(2)),
            Ok(ReplayPosition::Other(2))
        ));
        assert_eq!(replay.current_board(), &expected_board_2);
        assert!(matches!(
            replay.to_pos(ReplayPosition::End),
            Ok(ReplayPosition::End)
        ));
        assert_eq!(replay.current_board(), &expected_final_board);
        assert!(matches!(
            replay.to_pos(ReplayPosition::Other(1)),
            Ok(ReplayPosition::Other(1))
        ));
        assert_eq!(replay.current_board(), &expected_board_1);

        assert!(replay.to_pos(ReplayPosition::Other(5)).is_err());
    }
}
