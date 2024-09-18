use core::fmt;
use std::{
    fmt::{Debug, Display, Formatter},
    ops::{Index, IndexMut},
    slice::{Chunks, ChunksMut, Iter, IterMut},
};

use serde::{Deserialize, Serialize};

impl<T> From<&Board<T>> for Vec<Vec<T>>
where
    T: Copy,
{
    fn from(value: &Board<T>) -> Self {
        let mut return_board: Vec<Vec<T>> = Vec::new();
        for r in 0..value.rows() {
            let mut row = Vec::new();
            for c in 0..value.cols() {
                row.push(value[BoardPoint { row: r, col: c }]);
            }
            return_board.push(row);
        }
        return_board
    }
}

impl<T> From<Board<T>> for Vec<Vec<T>>
where
    T: Copy,
{
    fn from(value: Board<T>) -> Self {
        (&value).into()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct Board<T> {
    rows: usize,
    cols: usize,
    board: Vec<T>,
}

impl<T: Debug> Debug for Board<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let rows = (0..self.rows)
            .collect::<Vec<usize>>()
            .iter()
            .map(|row| {
                let row_slice = &self.board[(row * self.cols)..(row * self.cols + self.cols)];
                let row_flat = row_slice
                    .iter()
                    .fold(String::new(), |acc, item| acc + &format!("{:?}", item));
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

impl<T: Display> Display for Board<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let rows = (0..self.rows)
            .collect::<Vec<usize>>()
            .iter()
            .map(|row| {
                let row_slice = &self.board[(row * self.cols)..(row * self.cols + self.cols)];
                let row_flat = row_slice
                    .iter()
                    .fold(String::new(), |acc, item| acc + &format!("{}", item));
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

impl<T> Index<&BoardPoint> for Board<T> {
    type Output = T;

    fn index(&self, point: &BoardPoint) -> &Self::Output {
        let index = point.row * self.cols + point.col;
        &self.board[index]
    }
}

impl<T> IndexMut<&BoardPoint> for Board<T> {
    fn index_mut(&mut self, point: &BoardPoint) -> &mut Self::Output {
        let index = point.row * self.cols + point.col;
        &mut self.board[index]
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

    pub fn from_vec(vec: Vec<Vec<T>>) -> Self {
        let rows = vec.len();
        let cols = vec[0].len();
        Board {
            rows,
            cols,
            board: vec.into_iter().flatten().collect(),
        }
    }

    pub fn point_from_index(&self, index: usize) -> BoardPoint {
        BoardPoint {
            row: index / self.cols,
            col: index % self.cols,
        }
    }

    pub fn index_from_point(&self, point: BoardPoint) -> usize {
        point.row & (self.cols + point.col)
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

    pub fn rows_iter(&self) -> Chunks<T> {
        self.board.chunks(self.cols)
    }

    pub fn rows_iter_mut(&mut self) -> ChunksMut<T> {
        self.board.chunks_mut(self.cols)
    }

    pub fn iter(&self) -> Iter<'_, T> {
        self.board.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.board.iter_mut()
    }

    pub fn is_in_bounds(&self, point: BoardPoint) -> bool {
        point.row < self.rows && point.col < self.cols
    }

    pub fn neighbors(&self, point: &BoardPoint) -> Vec<BoardPoint> {
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
            if row < self.rows - 1 {
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
            if row < self.rows - 1 {
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

fn unsigned_diff<T>(first: T, second: T) -> usize
where
    T: Into<usize>,
{
    let first = first.into();
    let second = second.into();
    if first >= second {
        first - second
    } else {
        second - first
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BoardPoint {
    pub row: usize,
    pub col: usize,
}

impl BoardPoint {
    pub(crate) fn is_neighbor(&self, p2: &BoardPoint) -> bool {
        if self == p2 {
            // not neighbor to self
            return false;
        }
        unsigned_diff(self.row, p2.row) <= 1 && unsigned_diff(self.col, p2.col) <= 1
    }
}
