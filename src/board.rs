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

impl<T> Index<usize> for Board<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.board[index]
    }
}

impl<T> IndexMut<usize> for Board<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
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
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BoardPoint {
    pub row: usize,
    pub col: usize,
}

pub fn neighbors(index: usize, rows: usize, cols: usize) -> Vec<usize> {
    let mut neighbors = Vec::<usize>::new();

    let row = index / cols;
    let col = index % cols;
    if col > 0 {
        neighbors.push(index - 1);
        if row > 0 {
            neighbors.push(index - 1 - cols);
        }
        if row < cols - 1 {
            neighbors.push(index - 1 + cols);
        }
    }
    if col < cols - 1 {
        neighbors.push(index + 1);
        if row > 0 {
            neighbors.push(index + 1 - cols);
        }
        if row < cols - 1 {
            neighbors.push(index + 1 + cols);
        }
    }
    if row > 0 {
        neighbors.push(index - cols);
    }
    if row < rows - 1 {
        neighbors.push(index + cols);
    }
    neighbors
}
