#![allow(dead_code, unused_mut, unused_variables)]
use std::collections::{HashMap, HashSet};

// remove when done
use anyhow::{bail, Result};

use super::{MinesweeperReplay, ReplayPosition, Replayable};
use crate::{
    board::{Board, BoardPoint},
    cell::Cell,
    game::PlayOutcome,
};

#[derive(Debug, Clone, Copy)]
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

impl AnalysisCell {
    fn decrement(&mut self) {
        if let Self::Revealed(c) = self {
            *self = Self::Revealed(c.decrement());
        }
    }
}

struct AnalysisResult {
    guaranteed_plays: Vec<(BoardPoint, AnalyzedCell)>,
    found_fifty_fiftys: Vec<(BoardPoint, BoardPoint)>,
}

pub struct MinesweeperAnalysis {
    current_board: Board<Option<AnalyzedCell>>,
    log: Vec<Vec<(BoardPoint, Option<AnalyzedCell>)>>,
    current_pos: usize,
}

impl MinesweeperAnalysis {
    pub fn from_replay(replay: &mut MinesweeperReplay) -> Self {
        let mut analysis_board = Board::new(
            replay.current_board.rows(),
            replay.current_board.cols(),
            AnalysisCell::Hidden(AnalyzedCell::Undetermined),
        );
        let mut log: Vec<Vec<(BoardPoint, Option<AnalyzedCell>)>> =
            vec![Vec::new(); replay.log.len()];
        // save start_pos so we can return
        let start_pos = replay.current_pos;
        let _ = replay.to_pos(0);
        // loop over replay, updating log
        for i in 1..log.len() + 1 {
            let _ = replay.advance();
            let current_play = &replay.log[i].1;
            let new_revealed = match current_play {
                PlayOutcome::Success(v) => v,
                PlayOutcome::Failure(oc) => &vec![oc.clone()],
                PlayOutcome::Victory(v) => v,
                PlayOutcome::Flag(_) => continue,
            };
            let current_log_entry = &mut log[i];
            new_revealed.iter().for_each(|(bp, rc)| {
                let bp = *bp;
                // if previously analyzed, remove analysis state because it's now revealed
                if !matches!(
                    analysis_board[bp],
                    AnalysisCell::Hidden(AnalyzedCell::Undetermined)
                ) {
                    current_log_entry.push((bp.clone(), None));
                }
                analysis_board[bp] = AnalysisCell::Revealed(rc.contents);
            });
            let points_to_analyze = new_revealed
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
                .collect();
            Self::analyze_cells(
                points_to_analyze,
                &mut analysis_board,
                current_log_entry,
                HashMap::new(),
            );
            let current_board = replay.current_board();
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
        let _ = replay.to_pos(start_pos);
        let _ = analysis.to_pos(start_pos);
        analysis
    }

    fn analyze_cells(
        points_to_analyze: Vec<BoardPoint>,
        analysis_board: &mut Board<AnalysisCell>,
        current_log_entry: &mut Vec<(BoardPoint, Option<AnalyzedCell>)>,
        fifty_fiftys: HashMap<BoardPoint, BoardPoint>,
    ) {
        let mut fifty_fiftys = fifty_fiftys;
        let mut has_updates = false;
        let mut points_to_reanalyze = HashSet::new();
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
                    todo!()
                });
            res.guaranteed_plays.into_iter().for_each(|(point, ac)| {
                fifty_fiftys.remove(&point);
                analysis_board[point] = AnalysisCell::Hidden(ac);
                current_log_entry.push((point, Some(ac)));
                // add neighbors to points_to_reanalyze
                todo!()
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

    fn perform_checks(
        point: BoardPoint,
        analysis_board: &Board<AnalysisCell>,
        fifty_fiftys: &HashMap<BoardPoint, BoardPoint>,
    ) -> AnalysisResult {
        // get number of mines
        // get revealed points
        // get undetermined points
        todo!()
    }
}

impl Replayable for MinesweeperAnalysis {
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
        play.iter().for_each(|(bp, ac)| {
            self.current_board[*bp] = ac.clone();
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
        undo_play
            .iter()
            .filter(|(_, ac)| ac.is_some())
            .for_each(|(bp, ac)| {
                self.current_board[*bp] = None;
            });
        if self.current_pos > 0 {
            let redo_play = &self.log[self.current_pos - 1];
            redo_play.iter().for_each(|(bp, ac)| {
                self.current_board[*bp] = ac.clone();
            });
        }
        Ok(self.current_pos())
    }
}
