use core::{fmt, panic};
use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
};

use tinyvec::{array_vec, ArrayVec};

use crate::{
    board::{Board, BoardPoint},
    cell::{Cell, PlayerCell},
    upair::UnorderedPair,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalyzedCell {
    Mine,
    Empty,
    Undetermined,
}

impl Default for AnalyzedCell {
    fn default() -> Self {
        Self::Undetermined
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnalysisCell {
    Hidden(AnalyzedCell),
    Revealed(Cell),
}

impl Display for AnalysisCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AnalysisCell::Hidden(AnalyzedCell::Undetermined) => write!(f, "-"),
            AnalysisCell::Hidden(AnalyzedCell::Empty) => write!(f, "c"),
            AnalysisCell::Hidden(AnalyzedCell::Mine) => write!(f, "m"),
            AnalysisCell::Revealed(Cell::Mine) => write!(f, "M"),
            AnalysisCell::Revealed(Cell::Empty(x)) => write!(f, "{x}"),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnalysisUpdate {
    pub point: BoardPoint,
    pub from: Option<AnalyzedCell>,
    pub to: Option<AnalyzedCell>,
}

pub struct MinesweeperAnalysis {
    analysis_board: Board<AnalysisCell>,
    fifty_fiftys: Vec<UnorderedPair<BoardPoint>>,
}

impl MinesweeperAnalysis {
    pub fn init(board: &Board<PlayerCell>) -> Self {
        let mut analysis_board = Board::new(
            board.rows(),
            board.cols(),
            AnalysisCell::Hidden(AnalyzedCell::Undetermined),
        );
        let mut revealed_mines = Vec::new();
        board.rows_iter().enumerate().for_each(|(row, vec)| {
            vec.iter().enumerate().for_each(|(col, cell)| match cell {
                PlayerCell::Revealed(c) => {
                    let point = BoardPoint { row, col };
                    if matches!(c.contents, Cell::Mine) {
                        revealed_mines.push(point);
                    }
                    analysis_board[point] = AnalysisCell::Revealed(c.contents);
                }
                PlayerCell::Hidden(_) => {}
            });
        });
        revealed_mines.iter().for_each(|point| {
            analysis_board.neighbors(point).iter().for_each(|nbp| {
                if let AnalysisCell::Revealed(c) = analysis_board[nbp] {
                    // reduce neighboring cell numbers
                    analysis_board[nbp] = AnalysisCell::Revealed(c.decrement());
                }
            });
        });
        Self {
            analysis_board,
            fifty_fiftys: Vec::new(),
        }
    }

    pub fn analyze_board(&mut self) -> Vec<AnalysisUpdate> {
        let points_to_analyze = self
            .analysis_board
            .rows_iter()
            .enumerate()
            .flat_map(|(row, vec)| {
                vec.iter()
                    .enumerate()
                    .filter_map(|(col, _)| {
                        let bp = BoardPoint { row, col };
                        if matches!(
                            self.analysis_board[bp],
                            AnalysisCell::Revealed(Cell::Empty(_))
                        ) && self.has_undetermined_neighbor(&bp)
                        {
                            Some(bp)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        self.analyze_cells(points_to_analyze)
    }

    pub fn analyze_cells(&mut self, points_to_analyze: Vec<BoardPoint>) -> Vec<AnalysisUpdate> {
        let mut analysis_changes = Vec::new();
        let mut has_updates = false;
        let mut points_to_reanalyze = points_to_analyze.iter().copied().collect::<HashSet<_>>();

        points_to_analyze.into_iter().for_each(|bp| {
            let res = perform_checks(&bp, &self.analysis_board, &self.fifty_fiftys);
            if res.found_fifty_fiftys.is_some() || !res.guaranteed_plays.is_empty() {
                has_updates = true;
            }

            if let Some(pair) = res.found_fifty_fiftys {
                let point1 = pair.ref_a();
                let point2 = pair.ref_b();
                self.fifty_fiftys.push(pair);
                // add neighbors to points_to_reanalyze
                self.analysis_board
                    .neighbors(point1)
                    .iter()
                    .for_each(|nbp| {
                        let _ = points_to_reanalyze.insert(*nbp);
                    });
                self.analysis_board
                    .neighbors(point2)
                    .iter()
                    .for_each(|nbp| {
                        let _ = points_to_reanalyze.insert(*nbp);
                    });
            }
            let mut guaranteed_plays = res.guaranteed_plays;
            while !guaranteed_plays.is_empty() {
                // make sure we haven't handled this cell already
                let handle_plays = guaranteed_plays
                    .into_iter()
                    .filter(|(p, _)| {
                        !analysis_changes
                            .iter()
                            .any(|au: &AnalysisUpdate| &au.point == p)
                    })
                    .collect::<ArrayVec<[(BoardPoint, AnalyzedCell); 8]>>();
                // reset the array
                guaranteed_plays = array_vec!([(BoardPoint, AnalyzedCell); 8]);
                // handle the plays
                handle_plays.into_iter().for_each(|(point, ac)| {
                    while let Some(i) = self.fifty_fiftys.iter().enumerate().find_map(|(i, p)| {
                        if *p.ref_a() == point || *p.ref_b() == point {
                            Some(i)
                        } else {
                            None
                        }
                    }) {
                        let other = if self.fifty_fiftys[i].ref_a() == &point {
                            *self.fifty_fiftys[i].ref_b()
                        } else {
                            *self.fifty_fiftys[i].ref_a()
                        };
                        let guaranteed = match ac {
                            AnalyzedCell::Mine => AnalyzedCell::Empty,
                            AnalyzedCell::Empty => AnalyzedCell::Mine,
                            AnalyzedCell::Undetermined => panic!("This shouldn't be possible"),
                        };
                        let new_play = (other, guaranteed);
                        if !guaranteed_plays.contains(&new_play) {
                            guaranteed_plays.push(new_play);
                        }
                        self.fifty_fiftys.remove(i);
                    }
                    let from = match self.analysis_board[point] {
                        AnalysisCell::Hidden(AnalyzedCell::Empty) => Some(AnalyzedCell::Empty),
                        AnalysisCell::Hidden(AnalyzedCell::Mine) => Some(AnalyzedCell::Mine),
                        _ => None,
                    };
                    self.analysis_board[point] = AnalysisCell::Hidden(ac);
                    analysis_changes.push(AnalysisUpdate {
                        point,
                        from,
                        to: Some(ac),
                    });
                    // add neighbors to points_to_reanalyze
                    self.analysis_board
                        .neighbors(&point)
                        .iter()
                        .for_each(|nbp| {
                            if matches!(ac, AnalyzedCell::Mine) {
                                if let AnalysisCell::Revealed(c) = self.analysis_board[nbp] {
                                    // reduce neighboring cell numbers
                                    self.analysis_board[nbp] =
                                        AnalysisCell::Revealed(c.decrement());
                                }
                            }
                            let _ = points_to_reanalyze.insert(*nbp);
                        });
                });
            }
        });
        if !has_updates {
            return analysis_changes;
        }
        let points_to_reanalyze = points_to_reanalyze
            .into_iter()
            .filter(|point| {
                let cell = self.analysis_board[point];
                matches!(cell, AnalysisCell::Revealed(Cell::Empty(_)))
                    && self
                        .analysis_board
                        .neighbors(point)
                        .iter()
                        .map(|&nbp| self.analysis_board[nbp])
                        .any(|c| matches!(c, AnalysisCell::Hidden(AnalyzedCell::Undetermined)))
            })
            .collect();
        let mut recursive_changes = self.analyze_cells(points_to_reanalyze);
        analysis_changes.append(&mut recursive_changes);
        analysis_changes
    }

    pub fn apply_update(&mut self, point: &BoardPoint, cell: Cell) -> Option<AnalysisUpdate> {
        let mut ret = None;
        if !matches!(
            self.analysis_board[point],
            AnalysisCell::Hidden(AnalyzedCell::Undetermined)
        ) {
            let from = match self.analysis_board[point] {
                AnalysisCell::Hidden(AnalyzedCell::Empty) => Some(AnalyzedCell::Empty),
                AnalysisCell::Hidden(AnalyzedCell::Mine) => Some(AnalyzedCell::Mine),
                _ => None,
            };
            ret = Some(AnalysisUpdate {
                point: *point,
                from,
                to: None,
            });
        }
        let mut cell = cell;
        match cell {
            Cell::Empty(_) => {
                // reduce newly revealed cell by the number of known mines
                self.analysis_board
                    .neighbors(point)
                    .iter()
                    .filter(|&np| self.is_mine(np))
                    .for_each(|_| cell = cell.decrement());
            }
            Cell::Mine => {
                if !self.is_mine(point) {
                    // we now know this is a mine so we reduce existing revealed cells
                    let empty_neighbors = self
                        .analysis_board
                        .neighbors(point)
                        .into_iter()
                        .filter_map(|np| match self.analysis_board[np] {
                            AnalysisCell::Revealed(c) => Some((np, c)),
                            _ => None,
                        })
                        .collect::<ArrayVec<[(BoardPoint, Cell); 8]>>();
                    empty_neighbors.iter().for_each(|(np, c)| {
                        self.analysis_board[np] = AnalysisCell::Revealed(c.decrement());
                    });
                }
            }
        }
        while let Some(i) = self.fifty_fiftys.iter().enumerate().find_map(|(i, p)| {
            if p.ref_a() == point || p.ref_b() == point {
                Some(i)
            } else {
                None
            }
        }) {
            self.fifty_fiftys.remove(i);
        }
        self.analysis_board[point] = AnalysisCell::Revealed(cell);
        ret
    }

    pub(crate) fn has_undetermined_neighbor(&self, point: &BoardPoint) -> bool {
        self.analysis_board.neighbors(point).iter().any(|&nbp| {
            matches!(
                self.analysis_board[nbp],
                AnalysisCell::Hidden(AnalyzedCell::Undetermined)
            )
        })
    }
    pub(crate) fn is_empty(&self, point: &BoardPoint) -> bool {
        matches!(
            self.analysis_board[point],
            AnalysisCell::Revealed(Cell::Empty(_))
        )
    }

    pub(crate) fn is_mine(&self, point: &BoardPoint) -> bool {
        matches!(
            self.analysis_board[point],
            AnalysisCell::Revealed(Cell::Mine)
        ) || matches!(
            self.analysis_board[point],
            AnalysisCell::Hidden(AnalyzedCell::Mine)
        )
    }

    pub(crate) fn neighbors(&self, point: &BoardPoint) -> ArrayVec<[BoardPoint; 8]> {
        self.analysis_board.neighbors(point)
    }
}

fn neighbor_info(
    point: &BoardPoint,
    analysis_board: &Board<AnalysisCell>,
) -> (ArrayVec<[BoardPoint; 8]>, ArrayVec<[BoardPoint; 8]>) {
    let neighbors = analysis_board.neighbors(point);
    neighbors.iter().fold(
        (array_vec!([BoardPoint; 8]), array_vec!([BoardPoint; 8])),
        |(mut revealed_points, mut undetermined_points), p| {
            let ncell = analysis_board[p];
            match ncell {
                AnalysisCell::Hidden(AnalyzedCell::Undetermined) => undetermined_points.push(*p),
                AnalysisCell::Revealed(Cell::Empty(_)) => revealed_points.push(*p),
                _ => {}
            };
            (revealed_points, undetermined_points)
        },
    )
}

fn cell_to_num(cell: AnalysisCell) -> usize {
    (if let AnalysisCell::Revealed(Cell::Empty(x)) = cell {
        x
    } else {
        panic!("How did we get here")
    }) as usize
}

#[derive(Debug)]
struct AnalysisResult {
    guaranteed_plays: ArrayVec<[(BoardPoint, AnalyzedCell); 8]>,
    found_fifty_fiftys: Option<UnorderedPair<BoardPoint>>,
}

fn perform_checks(
    point: &BoardPoint,
    analysis_board: &Board<AnalysisCell>,
    fifty_fiftys: &[UnorderedPair<BoardPoint>],
) -> AnalysisResult {
    let cell = analysis_board[point];
    assert!(matches!(cell, AnalysisCell::Revealed(Cell::Empty(_))));

    let find_fifty_fifty_pairs = move |undetermined_points: &ArrayVec<[BoardPoint; 8]>| {
        fifty_fiftys
            .iter()
            .filter(|pair| {
                undetermined_points.contains(pair.ref_a())
                    && undetermined_points.contains(pair.ref_b())
            })
            .copied()
            .collect::<ArrayVec<[UnorderedPair<BoardPoint>; 8]>>()
    };

    let mut analysis_result = AnalysisResult {
        guaranteed_plays: array_vec!([(BoardPoint, AnalyzedCell); 8]),
        found_fifty_fiftys: None,
    };

    let (revealed_points, undetermined_points) = neighbor_info(point, analysis_board);

    let cell_num = cell_to_num(cell);
    if cell_num == 0 {
        // all mines satisfied, reveal others
        analysis_result.guaranteed_plays.append(
            &mut undetermined_points
                .into_iter()
                .map(|p| (p, AnalyzedCell::Empty))
                .collect(),
        );
        return analysis_result;
    }

    let num_undetermined = undetermined_points.len();
    if cell_num == num_undetermined {
        // all undetermined neighbors must be mines
        analysis_result.guaranteed_plays.append(
            &mut undetermined_points
                .into_iter()
                .map(|p| (p, AnalyzedCell::Mine))
                .collect(),
        );
        return analysis_result;
    }

    // it should be impossible for cell_num to be greater than num_undetermined
    // cells
    assert!(cell_num < num_undetermined);

    let fifty_fifty_pairs = find_fifty_fifty_pairs(&undetermined_points);
    let (non_fifty_fiftys, fifty_fifty_points) = undetermined_points.iter().fold(
        (array_vec!([BoardPoint; 8]), array_vec!([BoardPoint; 8])),
        |(mut non_fifty_fiftys, mut fifty_fifty_points), p| {
            if fifty_fifty_pairs
                .iter()
                .any(|pair| p == pair.ref_a() || p == pair.ref_b())
            {
                fifty_fifty_points.push(*p);
            } else {
                non_fifty_fiftys.push(*p);
            }
            (non_fifty_fiftys, fifty_fifty_points)
        },
    );
    let num_unique_fifty_fiftys = fifty_fifty_points.len() / 2;
    let exact_unique_fifty_fiftys = fifty_fifty_points.len() % 2 == 0;

    if cell_num == 1 && fifty_fifty_points.len() == 3 {
        // special case - overlapping 5050s next to 1
        analysis_result.guaranteed_plays.append(
            &mut fifty_fifty_points
                .into_iter()
                .map(|p| {
                    let overlap = fifty_fifty_pairs
                        .iter()
                        .filter(|up| up.ref_a() == &p || up.ref_b() == &p)
                        .count()
                        > 1;
                    if overlap {
                        (p, AnalyzedCell::Mine)
                    } else {
                        (p, AnalyzedCell::Empty)
                    }
                })
                .collect(),
        );
        return analysis_result;
    }

    if exact_unique_fifty_fiftys
        && cell_num - num_unique_fifty_fiftys == 1
        && non_fifty_fiftys.len() == 2
    {
        // new 5050 found common case
        analysis_result.found_fifty_fiftys =
            Some(UnorderedPair::new(non_fifty_fiftys[0], non_fifty_fiftys[1]));
        return analysis_result;
    }

    if cell_num == 2
        && undetermined_points.len() == 4
        && (fifty_fifty_points.len() == 3 || fifty_fifty_pairs.len() == 3)
    {
        // check for new 5050s special case
        let new_fifty_fifty = fifty_fifty_pairs.iter().find_map(|up| {
            let other = undetermined_points
                .iter()
                .copied()
                .filter(|p| p != up.ref_a() && p != up.ref_b())
                .collect::<ArrayVec<[BoardPoint; 2]>>();
            if other.len() == 2 {
                let pair = UnorderedPair::new(other[0], other[1]);
                if !fifty_fifty_pairs.contains(&pair) {
                    Some(pair)
                } else {
                    None
                }
            } else {
                None
            }
        });
        if new_fifty_fifty.is_some() {
            analysis_result.found_fifty_fiftys = new_fifty_fifty;
            return analysis_result;
        }
    }

    if cell_num == num_unique_fifty_fiftys && !non_fifty_fiftys.is_empty() {
        // all non-5050 cells are guaranteed plays
        analysis_result.guaranteed_plays.append(
            &mut non_fifty_fiftys
                .into_iter()
                .map(|p| (p, AnalyzedCell::Empty))
                .collect(),
        );
        return analysis_result;
    }

    if exact_unique_fifty_fiftys && cell_num - num_unique_fifty_fiftys == non_fifty_fiftys.len() {
        // all non-5050 cells are guaranteed mine
        analysis_result.guaranteed_plays.append(
            &mut non_fifty_fiftys
                .into_iter()
                .map(|p| (p, AnalyzedCell::Mine))
                .collect(),
        );
        return analysis_result;
    }

    for rp in revealed_points.iter() {
        let AnalysisCell::Revealed(Cell::Empty(r_num)) = analysis_board[rp] else {
            continue;
        };
        let r_num = r_num as usize;
        let other_undetermined = undetermined_points
            .iter()
            .filter(|p| !p.is_neighbor(rp))
            .copied()
            .collect::<ArrayVec<[BoardPoint; 8]>>();
        let num_other = other_undetermined.len();

        if num_other > 0 && cell_num > r_num && cell_num - r_num == num_other {
            // other_undetermined must be mines because rp's neighbors can't contain enough
            let mut guaranteed_mines = other_undetermined
                .into_iter()
                .map(|p| (p, AnalyzedCell::Mine))
                .collect();
            analysis_result
                .guaranteed_plays
                .append(&mut guaranteed_mines);
            return analysis_result;
        }

        let other_ff = other_undetermined
            .iter()
            .filter(|p| fifty_fifty_points.contains(p))
            .count();
        let all_other_ff = other_ff == other_undetermined.len();
        let other_mines = other_ff / 2;
        // rp's neighboring mines must be in undetermined points to satisfy current cell
        // therefore, rp's other neighbors can't be mines
        if all_other_ff && cell_num - other_mines == r_num {
            let mut rp_undetermined_other = analysis_board
                .neighbors(rp)
                .into_iter()
                .filter(|p| {
                    matches!(
                        analysis_board[p],
                        AnalysisCell::Hidden(AnalyzedCell::Undetermined)
                    )
                })
                .filter(|p| !undetermined_points.contains(p))
                .map(|p| (p, AnalyzedCell::Empty))
                .collect::<ArrayVec<[(BoardPoint, AnalyzedCell); 8]>>();
            if rp_undetermined_other.is_empty() {
                continue;
            }
            analysis_result
                .guaranteed_plays
                .append(&mut rp_undetermined_other);
            return analysis_result;
        }
    }

    // find all revealed "1"s with 2 or more undetermined cells as neighbors - treat as 5050
    let mut seen = array_vec!([BoardPoint; 8] => *point);
    let local_ff_points = revealed_points
        .into_iter()
        .filter(|p| matches!(analysis_board[p], AnalysisCell::Revealed(Cell::Empty(1))))
        .filter(|p| {
            let neighbors = undetermined_points
                .iter()
                .filter(|&p2| !seen.contains(p2))
                .filter(|&p2| p.is_neighbor(p2))
                .copied()
                .collect::<ArrayVec<[BoardPoint; 4]>>();
            if neighbors.len() >= 2 {
                neighbors.into_iter().for_each(|p| seen.push(p));
                true
            } else {
                false
            }
        })
        .collect::<ArrayVec<[BoardPoint; 8]>>();
    let mut not_ff = undetermined_points
        .iter()
        .filter(|p| !local_ff_points.iter().any(|p2| p.is_neighbor(p2)))
        .copied()
        .map(|p| (p, AnalyzedCell::Mine))
        .collect::<ArrayVec<[(BoardPoint, AnalyzedCell); 8]>>();

    if cell_num > num_undetermined / 2
        && !local_ff_points.is_empty()
        && cell_num - local_ff_points.len() == 1
        && not_ff.len() == 1
    {
        // all non-local-5050s are guaranteed mines
        analysis_result.guaranteed_plays.append(&mut not_ff);
        return analysis_result;
    };
    // exhausted all strategies
    analysis_result
}

// TODO - write unit tests
#[cfg(test)]
mod test {
    use super::*;

    fn visual_to_board(sboard: &str) -> Board<AnalysisCell> {
        let board = sboard
            .trim()
            .lines()
            .map(|row| {
                row.trim()
                    .chars()
                    .map(|c| match c {
                        '0' => AnalysisCell::Revealed(Cell::Empty(0)),
                        '1' => AnalysisCell::Revealed(Cell::Empty(1)),
                        '2' => AnalysisCell::Revealed(Cell::Empty(2)),
                        '3' => AnalysisCell::Revealed(Cell::Empty(3)),
                        '4' => AnalysisCell::Revealed(Cell::Empty(4)),
                        '5' => AnalysisCell::Revealed(Cell::Empty(5)),
                        '6' => AnalysisCell::Revealed(Cell::Empty(6)),
                        '7' => AnalysisCell::Revealed(Cell::Empty(7)),
                        '8' => AnalysisCell::Revealed(Cell::Empty(8)),
                        'M' => AnalysisCell::Revealed(Cell::Mine),
                        'm' => AnalysisCell::Hidden(AnalyzedCell::Mine),
                        'c' => AnalysisCell::Hidden(AnalyzedCell::Empty),
                        _ => AnalysisCell::Hidden(AnalyzedCell::Undetermined),
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        Board::from_vec(board)
    }

    struct TestCase(MinesweeperAnalysis, Board<AnalysisCell>);

    #[test]
    fn complex_reveal() {
        let cases: Vec<TestCase> = vec![
            TestCase(
                MinesweeperAnalysis {
                    analysis_board: visual_to_board(
                        "
                    ----
                    --2-
                    --21
                    --3-
                    -2--
                    ",
                    ),
                    fifty_fiftys: vec![
                        UnorderedPair::new(
                            BoardPoint { row: 4, col: 2 },
                            BoardPoint { row: 4, col: 3 },
                        ),
                        UnorderedPair::new(
                            BoardPoint { row: 3, col: 3 },
                            BoardPoint { row: 4, col: 3 },
                        ),
                    ],
                },
                visual_to_board(
                    "
                    ----
                    -c2c
                    --10
                    --1m
                    -1mc
                    ",
                ),
            ),
            TestCase(
                MinesweeperAnalysis {
                    analysis_board: visual_to_board(
                        "
                    -100
                    -100
                    121m
                    ----
                    ",
                    ),
                    fifty_fiftys: vec![],
                },
                visual_to_board(
                    "
                    -100
                    -100
                    110m
                    -cmc
                    ",
                ),
            ),
            TestCase(
                MinesweeperAnalysis {
                    analysis_board: visual_to_board(
                        "
                    111m
                    --2-
                    --3-
                    --31
                    --2-
                    ",
                    ),
                    fifty_fiftys: vec![],
                },
                visual_to_board(
                    "
                    111m
                    --2-
                    --2-
                    -m21
                    --1-
                    ",
                ),
            ),
            TestCase(
                MinesweeperAnalysis {
                    analysis_board: visual_to_board(
                        "
                    -22-1
                    ----1
                    m1m1m
                    ",
                    ),
                    fifty_fiftys: vec![],
                },
                visual_to_board(
                    "
                    m12-1
                    c---1
                    m1m1m
                    ",
                ),
            ),
        ];
        for case in cases.into_iter() {
            let mut analysis_state = case.0;
            let final_expected = case.1;

            let _res = analysis_state.analyze_board();

            println!(
                "Expected:\n{}\nGot:\n{}\nFifty Fiftys:{:?}",
                final_expected, analysis_state.analysis_board, analysis_state.fifty_fiftys
            );

            analysis_state
                .analysis_board
                .rows_iter()
                .enumerate()
                .for_each(|(row, vec)| {
                    vec.iter().enumerate().for_each(|(col, c)| {
                        assert!(*c == final_expected[BoardPoint { row, col }]);
                    })
                });
        }
    }
}
