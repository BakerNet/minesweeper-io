use core::panic;
use std::collections::{HashMap, HashSet};

// remove when done
use anyhow::{bail, Result};

#[cfg(test)]
use super::test::*;
use super::{MinesweeperReplay, ReplayPosition, Replayable};
use crate::{
    board::{Board, BoardPoint},
    cell::Cell,
    game::PlayOutcome,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyzedCell {
    Mine,
    Empty,
    Undetermined,
}

#[derive(Debug, Clone, Copy)]
enum AnalysisCell {
    Hidden(AnalyzedCell),
    Revealed(Cell),
}

#[derive(Debug)]
struct AnalysisResult {
    guaranteed_plays: Vec<(BoardPoint, AnalyzedCell)>,
    found_fifty_fiftys: Vec<(BoardPoint, BoardPoint)>,
}

#[derive(Debug, Clone)]
struct LogEntry {
    from: Option<AnalyzedCell>,
    to: Option<AnalyzedCell>,
}

pub struct MinesweeperAnalysis {
    current_board: Board<Option<AnalyzedCell>>,
    log: Vec<Vec<(BoardPoint, LogEntry)>>,
    current_pos: usize,
}

impl MinesweeperAnalysis {
    pub fn from_replay(replay: &MinesweeperReplay) -> Self {
        let mut analysis_board = Board::new(
            replay.current_board.rows(),
            replay.current_board.cols(),
            AnalysisCell::Hidden(AnalyzedCell::Undetermined),
        );
        let mut log: Vec<Vec<(BoardPoint, LogEntry)>> = vec![Vec::new(); replay.log.len()];
        let mut fifty_fiftys = HashMap::new();
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
                if !matches!(
                    analysis_board[bp],
                    AnalysisCell::Hidden(AnalyzedCell::Undetermined)
                ) {
                    let from = match analysis_board[bp] {
                        AnalysisCell::Hidden(AnalyzedCell::Empty) => Some(AnalyzedCell::Empty),
                        AnalysisCell::Hidden(AnalyzedCell::Mine) => Some(AnalyzedCell::Mine),
                        _ => None,
                    };
                    current_log_entry.push((bp, LogEntry { from, to: None }));
                }
                analysis_board[bp] = AnalysisCell::Revealed(rc.contents);
            });
            let mut points_to_analyze = new_revealed
                .iter()
                .filter_map(|(bp, _)| {
                    let bp = *bp;
                    if analysis_board.neighbors(bp).iter().any(|&nbp| {
                        matches!(
                            analysis_board[nbp],
                            AnalysisCell::Hidden(AnalyzedCell::Undetermined)
                        )
                    }) {
                        Some(bp)
                    } else {
                        None
                    }
                })
                .filter(|bp| matches!(analysis_board[*bp], AnalysisCell::Revealed(Cell::Empty(_))))
                .collect::<Vec<_>>();
            if matches!(current_play, PlayOutcome::Failure(_)) {
                let recheck = analysis_board
                    .neighbors(new_revealed[0].0)
                    .into_iter()
                    .filter(|bp| {
                        matches!(analysis_board[*bp], AnalysisCell::Revealed(Cell::Empty(_)))
                    })
                    .filter(|bp| {
                        analysis_board.neighbors(*bp).iter().any(|&nbp| {
                            matches!(
                                analysis_board[nbp],
                                AnalysisCell::Hidden(AnalyzedCell::Undetermined)
                            )
                        })
                    })
                    .filter(|bp| !points_to_analyze.contains(bp))
                    .collect::<Vec<_>>();
                recheck
                    .into_iter()
                    .for_each(|bp| points_to_analyze.push(bp));
            }
            Self::analyze_cells(
                points_to_analyze,
                &mut analysis_board,
                current_log_entry,
                &mut fifty_fiftys,
            );
        }

        let mut analysis = Self {
            current_board: Board::new(
                replay.current_board.rows(),
                replay.current_board.cols(),
                None::<AnalyzedCell>,
            ),
            log,
            current_pos: 0,
        };
        let _ = analysis.to_pos(replay.current_pos());
        analysis
    }

    fn analyze_cells(
        points_to_analyze: Vec<BoardPoint>,
        analysis_board: &mut Board<AnalysisCell>,
        current_log_entry: &mut Vec<(BoardPoint, LogEntry)>,
        fifty_fiftys: &mut HashMap<BoardPoint, BoardPoint>,
    ) {
        let mut has_updates = false;
        let mut points_to_reanalyze = HashSet::new();

        let add_to_reanalyze_if_has_unrevealed_neighbors =
            |point: BoardPoint,
             analysis_board: &mut Board<AnalysisCell>,
             points_to_reanalyze: &mut HashSet<BoardPoint>| {
                let cell = analysis_board[point];
                if matches!(cell, AnalysisCell::Revealed(Cell::Empty(_)))
                    && analysis_board
                        .neighbors(point)
                        .iter()
                        .map(|&nbp| analysis_board[nbp])
                        .any(|c| matches!(c, AnalysisCell::Hidden(AnalyzedCell::Undetermined)))
                {
                    points_to_reanalyze.insert(point);
                }
            };

        points_to_analyze.into_iter().for_each(|bp| {
            let res = Self::perform_checks(bp, analysis_board, &fifty_fiftys);
            if !res.found_fifty_fiftys.is_empty() || !res.guaranteed_plays.is_empty() {
                has_updates = true;
            }

            res.found_fifty_fiftys
                .into_iter()
                .for_each(|(point1, point2)| {
                    fifty_fiftys.insert(point1, point2);
                    fifty_fiftys.insert(point2, point1);
                    // add neighbors to points_to_reanalyze
                    analysis_board.neighbors(point1).iter().for_each(|nbp| {
                        add_to_reanalyze_if_has_unrevealed_neighbors(
                            *nbp,
                            analysis_board,
                            &mut points_to_reanalyze,
                        )
                    });
                    analysis_board.neighbors(point2).iter().for_each(|nbp| {
                        add_to_reanalyze_if_has_unrevealed_neighbors(
                            *nbp,
                            analysis_board,
                            &mut points_to_reanalyze,
                        )
                    });
                });
            res.guaranteed_plays.into_iter().for_each(|(point, ac)| {
                let other = fifty_fiftys.remove(&point);
                if let Some(obp) = other {
                    fifty_fiftys.remove(&obp);
                }
                let from = match analysis_board[point] {
                    AnalysisCell::Hidden(AnalyzedCell::Empty) => Some(AnalyzedCell::Empty),
                    AnalysisCell::Hidden(AnalyzedCell::Mine) => Some(AnalyzedCell::Mine),
                    _ => None,
                };
                analysis_board[point] = AnalysisCell::Hidden(ac);
                current_log_entry.push((point, LogEntry { from, to: Some(ac) }));
                // add neighbors to points_to_reanalyze
                analysis_board.neighbors(point).iter().for_each(|nbp| {
                    add_to_reanalyze_if_has_unrevealed_neighbors(
                        *nbp,
                        analysis_board,
                        &mut points_to_reanalyze,
                    )
                });
            });
        });
        if !has_updates {
            return;
        }
        Self::analyze_cells(
            points_to_reanalyze.into_iter().collect(),
            analysis_board,
            current_log_entry,
            fifty_fiftys,
        )
    }

    fn neighbor_info(
        point: BoardPoint,
        analysis_board: &Board<AnalysisCell>,
    ) -> (usize, Vec<BoardPoint>, Vec<BoardPoint>) {
        let neighbors = analysis_board.neighbors(point);
        let (num_mines, revealed_points, undetermined_points) = neighbors.iter().fold(
            (0, Vec::new(), Vec::new()),
            |(mut num_mines, mut revealed_points, mut undetermined_points), &p| {
                let ncell = analysis_board[p];
                match ncell {
                    AnalysisCell::Hidden(AnalyzedCell::Undetermined) => undetermined_points.push(p),
                    AnalysisCell::Hidden(AnalyzedCell::Mine) => num_mines += 1,
                    AnalysisCell::Hidden(_) => {}
                    AnalysisCell::Revealed(Cell::Mine) => num_mines += 1,
                    AnalysisCell::Revealed(Cell::Empty(_)) => revealed_points.push(p),
                };
                (num_mines, revealed_points, undetermined_points)
            },
        );
        (num_mines, revealed_points, undetermined_points)
    }

    fn cell_to_num(cell: AnalysisCell) -> usize {
        (if let AnalysisCell::Revealed(Cell::Empty(x)) = cell {
            x
        } else {
            panic!("How did we get here")
        }) as usize
    }

    fn perform_checks(
        point: BoardPoint,
        analysis_board: &Board<AnalysisCell>,
        fifty_fiftys: &HashMap<BoardPoint, BoardPoint>,
    ) -> AnalysisResult {
        let cell = analysis_board[point];
        assert!(matches!(cell, AnalysisCell::Revealed(Cell::Empty(_))));

        let find_fifty_fifty_pairs = move |undetermined_points: &Vec<BoardPoint>| {
            undetermined_points.iter().fold(Vec::new(), |mut acc, p| {
                if let Some(p2) = fifty_fiftys.get(p) {
                    if undetermined_points.contains(p2) && !acc.contains(&(*p2, *p)) {
                        acc.push((*p, *p2));
                    }
                }
                acc
            })
        };

        let (num_mines, revealed_points, undetermined_points) =
            Self::neighbor_info(point, analysis_board);
        let fifty_fifty_pairs = find_fifty_fifty_pairs(&undetermined_points);
        let cell_num = Self::cell_to_num(cell) - num_mines;
        if cell_num == 0 {
            return AnalysisResult {
                guaranteed_plays: undetermined_points
                    .into_iter()
                    .map(|p| (p, AnalyzedCell::Empty))
                    .collect(),
                found_fifty_fiftys: Vec::new(),
            };
        }
        let num_undetermined = undetermined_points.len();
        if cell_num == num_undetermined {
            return AnalysisResult {
                guaranteed_plays: undetermined_points
                    .into_iter()
                    .map(|p| (p, AnalyzedCell::Mine))
                    .collect(),
                found_fifty_fiftys: Vec::new(),
            };
        }

        // it should be impossible for cell_num to be greater than num_undetermined
        // cells
        assert!(cell_num < num_undetermined);
        let num_fifty_fiftys = fifty_fifty_pairs.len();
        if num_fifty_fiftys * 2 == num_undetermined {
            // no further analysis is possilbe from this cell
            return AnalysisResult {
                guaranteed_plays: Vec::new(),
                found_fifty_fiftys: Vec::new(),
            };
        }
        if cell_num == num_fifty_fiftys {
            // all non-5050 cells are guaranteed plays
            return AnalysisResult {
                guaranteed_plays: undetermined_points
                    .into_iter()
                    .filter(|p| {
                        !fifty_fifty_pairs
                            .iter()
                            .any(|(ffp1, ffp2)| p == ffp1 || p == ffp2)
                    })
                    .map(|p| (p, AnalyzedCell::Empty))
                    .collect(),
                found_fifty_fiftys: Vec::new(),
            };
        }
        if cell_num == num_undetermined - num_fifty_fiftys {
            // all non-5050 cells are guaranteed mine
            return AnalysisResult {
                guaranteed_plays: undetermined_points
                    .into_iter()
                    .filter(|p| {
                        !fifty_fifty_pairs
                            .iter()
                            .any(|(ffp1, ffp2)| p == ffp1 || p == ffp2)
                    })
                    .map(|p| (p, AnalyzedCell::Mine))
                    .collect(),
                found_fifty_fiftys: Vec::new(),
            };
        }
        if cell_num == 1 && num_undetermined == 2 && num_fifty_fiftys == 0 {
            // new 5050 found
            return AnalysisResult {
                guaranteed_plays: Vec::new(),
                found_fifty_fiftys: vec![(undetermined_points[0], undetermined_points[1])],
            };
        }
        // // check for "1" next to 2 undetermined cells - it's a local 5050
        // // find all revealed with 2 undetermined cells as neighbors
        // // filter to ones that have value 1 - there should only be one
        // // rest are guaranteed mine
        let local_ff_points = revealed_points
            .into_iter()
            .filter(|p| matches!(analysis_board[*p], AnalysisCell::Revealed(Cell::Empty(1))))
            .filter(|p| {
                let neighbor_points = undetermined_points
                    .iter()
                    .filter(|&p2| p.is_neighbor(*p2))
                    .collect::<Vec<_>>();
                neighbor_points.len() == 2
            })
            .collect::<Vec<_>>();
        if !local_ff_points.is_empty()
            && local_ff_points.len() * 2 == num_undetermined - 1
            && cell_num > 1
        {
            let not_ff = undetermined_points
                .into_iter()
                .filter(|p| !local_ff_points.iter().any(|p2| p.is_neighbor(*p2)))
                .map(|p| (p, AnalyzedCell::Mine))
                .collect::<Vec<_>>();
            if not_ff.len() == 1 {
                return AnalysisResult {
                    guaranteed_plays: not_ff,
                    found_fifty_fiftys: Vec::new(),
                };
            }
        };
        // exhausted all strategies
        AnalysisResult {
            guaranteed_plays: Vec::new(),
            found_fifty_fiftys: Vec::new(),
        }
    }
}

impl MinesweeperAnalysis {
    pub fn current_board(&self) -> &Board<Option<AnalyzedCell>> {
        &self.current_board
    }
}

impl Replayable for MinesweeperAnalysis {
    fn len(&self) -> usize {
        self.log.len() + 1
    }

    fn current_pos(&self) -> ReplayPosition {
        ReplayPosition::from_pos(self.current_pos, self.len() - 1)
    }

    fn advance(&mut self) -> Result<ReplayPosition> {
        if self.current_pos == self.len() - 1 {
            bail!("Called next on end")
        }
        let play = &self.log[self.current_pos];
        play.iter().for_each(|(bp, entry)| {
            self.current_board[*bp] = entry.to;
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
        undo_play.iter().for_each(|(bp, entry)| {
            self.current_board[*bp] = entry.from;
        });
        if self.current_pos > 0 {
            let redo_play = &self.log[self.current_pos - 1];
            redo_play.iter().for_each(|(bp, entry)| {
                self.current_board[*bp] = entry.to;
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

        let replay = MinesweeperReplay::new(
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

        let mut analysis = MinesweeperAnalysis::from_replay(&replay);
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
