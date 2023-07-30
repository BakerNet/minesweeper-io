use core::fmt;
use std::{
    fmt::{Debug, Display, Formatter},
    ops::{Index, IndexMut},
    slice::Iter,
};

pub struct Board<T> {
    rows: usize,
    cols: usize,
    board: Vec<T>,
}

impl<T> Display for Board<T>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let rows = (0..self.rows)
            .collect::<Vec<usize>>()
            .iter()
            .map(|row| {
                let row_slice = &self.board[(row * self.cols)..(row * self.cols + self.cols)];
                let row_flat = row_slice
                    .into_iter()
                    .map(|item| format!("{:?}", item))
                    .collect::<String>();
                row_flat
            })
            .fold(String::new(), |mut acc, s| {
                acc.push_str(&format!("{}\n", s));
                acc
            });
        let row_trim = rows.trim_end();
        write!(f, "{}", row_trim)
    }
}

impl<T> Index<BoardPoint> for Board<T> {
    type Output = T;

    fn index(&self, point: BoardPoint) -> &Self::Output {
        let index = point.row * self.cols + point.col;
        &self.board[index]
    }
}

impl<T> IndexMut<BoardPoint> for Board<T> {
    fn index_mut(&mut self, point: BoardPoint) -> &mut Self::Output {
        let index = point.row * self.cols + point.col;
        &mut self.board[index]
    }
}

impl<T> Board<T> {
    pub fn new(rows: usize, cols: usize, item: T) -> Self
    where
        T: Clone,
    {
        let total = rows * cols;
        Board {
            rows,
            cols,
            board: vec![item; total],
        }
    }

    pub fn point_from_index(&self, index: usize) -> BoardPoint {
        BoardPoint {
            row: index / self.cols,
            col: index % self.rows,
        }
    }

    pub fn index_from_point(&self, point: BoardPoint) -> usize {
        point.row & self.cols + point.col
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn len(&self) -> usize {
        self.board.len()
    }

    pub fn is_empty(&self) -> bool {
        self.board.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.board.iter()
    }

    pub fn is_in_bounds(&self, point: BoardPoint) -> bool {
        point.row < self.rows && point.col < self.cols
    }

    pub fn neighbors(&self, point: BoardPoint) -> Vec<BoardPoint> {
        let mut neighbors = Vec::<BoardPoint>::new();

        let row = point.row;
        let col = point.col;
        if col > 0 {
            neighbors.push(BoardPoint { row, col: col - 1 });
            if row > 0 {
                neighbors.push(BoardPoint {
                    row: row - 1,
                    col: col - 1,
                });
            }
            if row < self.cols - 1 {
                neighbors.push(BoardPoint {
                    row: row + 1,
                    col: col - 1,
                });
            }
        }
        if col < self.cols - 1 {
            neighbors.push(BoardPoint { row, col: col + 1 });
            if row > 0 {
                neighbors.push(BoardPoint {
                    row: row - 1,
                    col: col + 1,
                });
            }
            if row < self.cols - 1 {
                neighbors.push(BoardPoint {
                    row: row + 1,
                    col: col + 1,
                });
            }
        }
        if row > 0 {
            neighbors.push(BoardPoint { row: row - 1, col });
        }
        if row < self.rows - 1 {
            neighbors.push(BoardPoint { row: row + 1, col });
        }
        neighbors
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BoardPoint {
    pub row: usize,
    pub col: usize,
}
