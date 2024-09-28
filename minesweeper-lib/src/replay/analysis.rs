use std::collections::HashSet;

// remove when done
use anyhow::{bail, Result};
use tinyvec::ArrayVec;

#[cfg(test)]
use super::test::*;
use super::{MinesweeperReplay, ReplayPosition, Replayable};
use crate::{
    analysis::{AnalysisUpdate, AnalyzedCell, MinesweeperAnalysis},
    board::{Board, BoardPoint},
    game::PlayOutcome,
};

pub struct MinesweeperReplayAnalysis {
    current_board: Board<Option<AnalyzedCell>>,
    log: Vec<Vec<AnalysisUpdate>>,
    current_pos: usize,
}

impl MinesweeperReplayAnalysis {
    pub fn from_replay(replay: &mut MinesweeperReplay) -> Self {
        let _ = replay.to_pos(ReplayPosition::Beginning);
        let mut analysis_state = MinesweeperAnalysis::init(replay.current_board());
        let mut log: Vec<Vec<AnalysisUpdate>> = vec![Vec::new(); replay.log.len()];

        // loop over replay, updating log
        for (i, current_log_entry) in log.iter_mut().enumerate() {
            let current_play = &replay.log[i].1;
            let new_revealed = match current_play {
                PlayOutcome::Success(v) => v,
                PlayOutcome::Failure(oc) => &vec![*oc],
                PlayOutcome::Victory(v) => v,
                PlayOutcome::Flag(_) => continue,
            };
            new_revealed.iter().for_each(|(bp, rc)| {
                let bp = *bp;
                // if previously analyzed, remove analysis state because it's now revealed
                let update = analysis_state.apply_update(&bp, rc.contents);
                if let Some(log_value) = update {
                    current_log_entry.push(log_value);
                }
            });
            let mut points_to_analyze = new_revealed
                .iter()
                .filter_map(|(bp, _)| {
                    if analysis_state.has_undetermined_neighbor(bp) {
                        Some(*bp)
                    } else {
                        None
                    }
                })
                .filter(|bp| analysis_state.is_empty(bp))
                .collect::<HashSet<_>>();
            let additional_points = points_to_analyze
                .iter()
                .flat_map(|p| {
                    analysis_state
                        .neighbors(p)
                        .into_iter()
                        .filter(|np| !points_to_analyze.contains(np))
                        .filter(|np| analysis_state.is_empty(np))
                        .filter(|np| analysis_state.has_undetermined_neighbor(np))
                        .collect::<ArrayVec<[BoardPoint; 8]>>()
                })
                .collect::<Vec<_>>();
            additional_points.into_iter().for_each(|p| {
                let _ = points_to_analyze.insert(p);
            });
            if matches!(current_play, PlayOutcome::Failure(_)) {
                let recheck = analysis_state
                    .neighbors(&new_revealed[0].0)
                    .into_iter()
                    .filter(|bp| !points_to_analyze.contains(bp))
                    .filter(|bp| analysis_state.is_empty(bp))
                    .filter(|bp| analysis_state.has_undetermined_neighbor(bp))
                    .collect::<ArrayVec<[BoardPoint; 8]>>();
                recheck.into_iter().for_each(|bp| {
                    let _ = points_to_analyze.insert(bp);
                });
            }
            let mut analysis_res =
                analysis_state.analyze_cells(points_to_analyze.into_iter().collect());
            current_log_entry.append(&mut analysis_res);
        }

        Self {
            current_board: Board::new(
                replay.current_board.rows(),
                replay.current_board.cols(),
                None::<AnalyzedCell>,
            ),
            log,
            current_pos: 0,
        }
    }

    pub fn current_board(&self) -> &Board<Option<AnalyzedCell>> {
        &self.current_board
    }
}

impl Replayable for MinesweeperReplayAnalysis {
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
        play.iter().for_each(|update| {
            self.current_board[update.point] = update.to;
        });
        self.current_pos += 1;
        Ok(self.current_pos())
    }

    fn rewind(&mut self) -> Result<ReplayPosition> {
        if self.current_pos == 0 {
            bail!("Called next on end")
        }
        self.current_pos -= 1;
        let undo_play = &self.log[self.current_pos];
        undo_play.iter().for_each(|update| {
            self.current_board[update.point] = update.from;
        });
        if self.current_pos > 0 {
            let redo_play = &self.log[self.current_pos - 1];
            redo_play.iter().for_each(|update| {
                self.current_board[update.point] = update.to;
            });
        }
        Ok(self.current_pos())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        cell::{HiddenCell, PlayerCell},
        game::{Action, Play},
    };

    use super::*;

    const PLAY_1_GUARANTEES: [(BoardPoint, AnalyzedCell); 2] = [
        (BoardPoint { row: 2, col: 3 }, AnalyzedCell::Empty),
        (BoardPoint { row: 3, col: 2 }, AnalyzedCell::Mine),
    ];
    const PLAY_3_GUARANTEES: [(BoardPoint, AnalyzedCell); 2] = [
        (BoardPoint { row: 3, col: 0 }, AnalyzedCell::Mine),
        (BoardPoint { row: 3, col: 1 }, AnalyzedCell::Empty),
    ];
    const PLAY_4_GUARANTEES: [(BoardPoint, AnalyzedCell); 2] = [
        (BoardPoint { row: 0, col: 3 }, AnalyzedCell::Mine),
        (BoardPoint { row: 1, col: 3 }, AnalyzedCell::Empty),
    ];

    #[test]
    fn test_analysis() {
        let expected_starting_board = Board::new(4, 4, None::<AnalyzedCell>);

        let mut expected_next_board = expected_starting_board.clone();

        // res of first play
        expected_next_board[PLAY_1_GUARANTEES[0].0] = Some(PLAY_1_GUARANTEES[0].1);
        expected_next_board[PLAY_1_GUARANTEES[1].0] = Some(PLAY_1_GUARANTEES[1].1);
        let expected_board_1 = expected_next_board.clone();

        // res of second play
        // flag has no effect on analysis
        let expected_board_2 = expected_next_board.clone();

        // res of third play
        // revealed cells unest analysis
        expected_next_board[PLAY_3_RES.0] = None;
        // info gained by revealing "2" at BoardPoint{row: 2, col: 3}
        expected_next_board[PLAY_3_GUARANTEES[0].0] = Some(PLAY_3_GUARANTEES[0].1);
        expected_next_board[PLAY_3_GUARANTEES[1].0] = Some(PLAY_3_GUARANTEES[1].1);
        let expected_board_3 = expected_next_board.clone();

        // res of final play
        // info gained by revealing mine where 5050 previously resided
        expected_next_board[PLAY_4_GUARANTEES[0].0] = Some(PLAY_4_GUARANTEES[0].1);
        expected_next_board[PLAY_4_GUARANTEES[1].0] = Some(PLAY_4_GUARANTEES[1].1);
        let expected_final_board = expected_next_board.clone();

        let mut replay = MinesweeperReplay::new(
            Board::new(4, 4, PlayerCell::Hidden(HiddenCell::Empty)),
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

        let mut analysis = MinesweeperReplayAnalysis::from_replay(&mut replay);
        assert_eq!(analysis.len(), replay.len());

        assert_eq!(analysis.current_board, Board::new(4, 4, None));

        // test advance
        assert!(matches!(analysis.advance(), Ok(ReplayPosition::Other(1))));
        assert_eq!(analysis.current_board(), &expected_board_1);
        assert!(matches!(analysis.advance(), Ok(ReplayPosition::Other(2))));
        assert_eq!(analysis.current_board(), &expected_board_2);
        assert!(matches!(analysis.advance(), Ok(ReplayPosition::Other(3))));
        assert_eq!(analysis.current_board(), &expected_board_3);
        assert!(matches!(analysis.advance(), Ok(ReplayPosition::End)));
        assert_eq!(analysis.current_board(), &expected_final_board);

        // should error on advance at end
        assert!(analysis.advance().is_err());

        // test rewind
        assert!(matches!(analysis.rewind(), Ok(ReplayPosition::Other(3))));
        assert_eq!(analysis.current_board(), &expected_board_3);
        assert!(matches!(analysis.rewind(), Ok(ReplayPosition::Other(2))));
        assert_eq!(analysis.current_board(), &expected_board_2);
        assert!(matches!(analysis.rewind(), Ok(ReplayPosition::Other(1))));
        assert_eq!(analysis.current_board(), &expected_board_1);
        assert!(matches!(analysis.rewind(), Ok(ReplayPosition::Beginning)));
        assert_eq!(analysis.current_board(), &expected_starting_board);

        // should error on rewind at beginning
        assert!(analysis.rewind().is_err());
    }
}
