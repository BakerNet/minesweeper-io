use std::ops::IndexMut;

pub struct Board<T> {
    rows: usize,
    cols: usize,
    board: Vec<T>,
}

impl<T> IndexMut<usize> for Board<T> {
    type Output = T;

    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.board[index]
    }
}

impl<T> Board<T> {
    pub fn point_from_index(&self, index: usize) -> BoardPoint {
        BoardPoint {
            row: index / self.cols,
            col: index % self.rows,
        }
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
