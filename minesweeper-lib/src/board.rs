use core::fmt;
use std::{
    fmt::{Debug, Display, Formatter},
    ops::{Index, IndexMut},
    slice::{Chunks, ChunksMut, Iter, IterMut},
};

use serde::{Deserialize, Serialize};
use tinyvec::{array_vec, ArrayVec};

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

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
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
                    .fold(String::new(), |acc, item| acc + &format!("{item:?}"));
                row_flat
            })
            .fold(String::new(), |mut acc, s| {
                acc.push_str(&format!("{s}\n"));
                acc
            });
        let row_trim = rows.trim_end();
        write!(f, "{row_trim}")
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
                    .fold(String::new(), |acc, item| acc + &format!("{item}"));
                row_flat
            })
            .fold(String::new(), |mut acc, s| {
                acc.push_str(&format!("{s}\n"));
                acc
            });
        let row_trim = rows.trim_end();
        write!(f, "{row_trim}")
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

    pub fn size(&self) -> usize {
        self.board.len()
    }

    pub fn is_empty(&self) -> bool {
        self.board.is_empty()
    }

    pub fn rows_iter(&self) -> Chunks<'_, T> {
        self.board.chunks(self.cols)
    }

    pub fn rows_iter_mut(&mut self) -> ChunksMut<'_, T> {
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

    pub fn neighbors(&self, point: &BoardPoint) -> ArrayVec<[BoardPoint; 8]> {
        let mut neighbors = array_vec!([BoardPoint; 8]);

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
    first.abs_diff(second)
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default, PartialOrd, Ord,
)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompactBoard {
    rows: usize,
    cols: usize,
    data: Vec<u8>,
}

impl CompactBoard {
    pub fn from_board<T>(board: &Board<T>) -> Self
    where
        T: CompactSerialize + Copy + PartialEq,
    {
        let mut data = Vec::new();
        let mut current_cell = None;
        let mut count = 0u8;

        for cell in board.iter() {
            match current_cell {
                None => {
                    current_cell = Some(*cell);
                    count = 1;
                }
                Some(ref prev_cell) if prev_cell == cell && count < 255 => {
                    count += 1;
                }
                Some(prev_cell) => {
                    // Write the previous run
                    data.push(prev_cell.to_compact_byte());
                    data.push(count);

                    // Start new run
                    current_cell = Some(*cell);
                    count = 1;
                }
            }
        }

        // Write final run
        if let Some(cell) = current_cell {
            data.push(cell.to_compact_byte());
            data.push(count);
        }

        CompactBoard {
            rows: board.rows(),
            cols: board.cols(),
            data,
        }
    }

    pub fn to_board<T>(&self) -> Board<T>
    where
        T: CompactSerialize + Copy,
    {
        let mut cells = Vec::with_capacity(self.rows * self.cols);

        for chunk in self.data.chunks(2) {
            if chunk.len() == 2 {
                let cell = T::from_compact_byte(chunk[0]);
                let count = chunk[1] as usize;
                for _ in 0..count {
                    cells.push(cell);
                }
            }
        }

        // Fill remaining cells with default if needed
        let expected_size = self.rows * self.cols;
        while cells.len() < expected_size {
            cells.push(T::from_compact_byte(0)); // Default to first variant
        }

        Board {
            rows: self.rows,
            cols: self.cols,
            board: cells,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }
}

pub trait CompactSerialize {
    fn to_compact_byte(&self) -> u8;
    fn from_compact_byte(byte: u8) -> Self;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::{HiddenCell, PlayerCell};

    #[test]
    fn test_compact_board_compression() {
        // Create a typical early-game board (mostly empty)
        let rows = 16;
        let cols = 30;
        let board = Board::new(rows, cols, PlayerCell::Hidden(HiddenCell::Empty));

        // Convert to compact format
        let compact = CompactBoard::from_board(&board);

        // Verify it round-trips correctly
        let restored: Board<PlayerCell> = compact.to_board();

        assert_eq!(board.rows(), restored.rows());
        assert_eq!(board.cols(), restored.cols());
        for (orig, rest) in board.iter().zip(restored.iter()) {
            assert_eq!(orig, rest);
        }

        // Test compression - should be much smaller than original
        let original_json = serde_json::to_string(&board).unwrap();
        let compact_json = serde_json::to_string(&compact).unwrap();

        println!("16x30 board - Original size: {} bytes", original_json.len());
        println!("16x30 board - Compact size: {} bytes", compact_json.len());
        println!(
            "16x30 board - Compression ratio: {:.2}%",
            (compact_json.len() as f64 / original_json.len() as f64) * 100.0
        );

        // Should achieve significant compression for homogeneous boards
        assert!(compact_json.len() < original_json.len() / 10);
    }

    #[test]
    fn test_large_multiplayer_board_compression() {
        // Test large multiplayer board (50x50)
        let rows = 50;
        let cols = 50;
        let board = Board::new(rows, cols, PlayerCell::Hidden(HiddenCell::Empty));

        let compact = CompactBoard::from_board(&board);
        let restored: Board<PlayerCell> = compact.to_board();

        assert_eq!(board.rows(), restored.rows());
        assert_eq!(board.cols(), restored.cols());

        let original_json = serde_json::to_string(&board).unwrap();
        let compact_json = serde_json::to_string(&compact).unwrap();

        println!("50x50 board - Original size: {} bytes", original_json.len());
        println!("50x50 board - Compact size: {} bytes", compact_json.len());
        println!(
            "50x50 board - Compression ratio: {:.2}%",
            (compact_json.len() as f64 / original_json.len() as f64) * 100.0
        );
        println!(
            "50x50 board - Bandwidth savings: {:.2}%",
            (1.0 - compact_json.len() as f64 / original_json.len() as f64) * 100.0
        );

        // Large boards should compress even better
        assert!(compact_json.len() < original_json.len() / 20);
    }

    #[test]
    fn test_compact_board_with_mixed_cells() {
        use crate::cell::{Cell, RevealedCell};

        let rows = 3;
        let cols = 3;
        let mut board = Board::new(rows, cols, PlayerCell::Hidden(HiddenCell::Empty));

        // Add some variety
        board[BoardPoint { row: 0, col: 0 }] = PlayerCell::Hidden(HiddenCell::Flag);
        board[BoardPoint { row: 1, col: 1 }] = PlayerCell::Revealed(RevealedCell {
            player: 0,
            contents: Cell::Empty(3),
        });
        board[BoardPoint { row: 2, col: 2 }] = PlayerCell::Hidden(HiddenCell::Mine);

        // Test round-trip
        let compact = CompactBoard::from_board(&board);
        let restored: Board<PlayerCell> = compact.to_board();

        assert_eq!(board.rows(), restored.rows());
        assert_eq!(board.cols(), restored.cols());
        for (orig, rest) in board.iter().zip(restored.iter()) {
            assert_eq!(orig, rest);
        }
    }

    #[test]
    fn test_compact_board_supports_12_players() {
        use crate::cell::{Cell, RevealedCell};

        let rows = 4;
        let cols = 3;
        let mut board = Board::new(rows, cols, PlayerCell::Hidden(HiddenCell::Empty));

        // Test all player IDs from 0 to 11 (12 players total)
        for player_id in 0..12 {
            let row = player_id / cols;
            let col = player_id % cols;
            board[BoardPoint { row, col }] = PlayerCell::Revealed(RevealedCell {
                player: player_id,
                contents: Cell::Empty((player_id % 9) as u8), // Use various content values 0-8
            });
        }

        // Test round-trip with all 12 players
        let compact = CompactBoard::from_board(&board);
        let restored: Board<PlayerCell> = compact.to_board();

        assert_eq!(board.rows(), restored.rows());
        assert_eq!(board.cols(), restored.cols());

        // Verify all player IDs are preserved correctly
        for player_id in 0..12 {
            let row = player_id / cols;
            let col = player_id % cols;
            let point = BoardPoint { row, col };

            match (&board[point], &restored[point]) {
                (
                    PlayerCell::Revealed(RevealedCell {
                        player: orig_player,
                        contents: orig_contents,
                    }),
                    PlayerCell::Revealed(RevealedCell {
                        player: rest_player,
                        contents: rest_contents,
                    }),
                ) => {
                    assert_eq!(
                        orig_player, rest_player,
                        "Player ID mismatch for player {}",
                        player_id
                    );
                    assert_eq!(
                        orig_contents, rest_contents,
                        "Contents mismatch for player {}",
                        player_id
                    );
                }
                _ => panic!("Cell type mismatch for player {}", player_id),
            }
        }

        println!("✓ Successfully tested all 12 players (0-11) with proper round-trip encoding");
    }

    #[test]
    fn test_compact_board_with_mines_and_high_values() {
        use crate::cell::{Cell, RevealedCell};

        let rows = 3;
        let cols = 3;
        let mut board = Board::new(rows, cols, PlayerCell::Hidden(HiddenCell::Empty));

        // Test various edge cases
        board[BoardPoint { row: 0, col: 0 }] = PlayerCell::Revealed(RevealedCell {
            player: 11,               // High player ID
            contents: Cell::Empty(8), // Max neighbor count
        });
        board[BoardPoint { row: 0, col: 1 }] = PlayerCell::Revealed(RevealedCell {
            player: 7,
            contents: Cell::Mine, // Mine
        });
        board[BoardPoint { row: 0, col: 2 }] = PlayerCell::Revealed(RevealedCell {
            player: 0,
            contents: Cell::Empty(7), // Previously problematic value
        });

        // Test round-trip
        let compact = CompactBoard::from_board(&board);
        let restored: Board<PlayerCell> = compact.to_board();

        assert_eq!(board.rows(), restored.rows());
        assert_eq!(board.cols(), restored.cols());
        for (orig, rest) in board.iter().zip(restored.iter()) {
            assert_eq!(orig, rest);
        }

        // Specifically verify the edge cases
        match &restored[BoardPoint { row: 0, col: 0 }] {
            PlayerCell::Revealed(RevealedCell {
                player: 11,
                contents: Cell::Empty(8),
            }) => {}
            other => panic!("Expected player 11 with Empty(8), got {:?}", other),
        }

        match &restored[BoardPoint { row: 0, col: 1 }] {
            PlayerCell::Revealed(RevealedCell {
                player: 7,
                contents: Cell::Mine,
            }) => {}
            other => panic!("Expected player 7 with Mine, got {:?}", other),
        }

        match &restored[BoardPoint { row: 0, col: 2 }] {
            PlayerCell::Revealed(RevealedCell {
                player: 0,
                contents: Cell::Empty(7),
            }) => {}
            other => panic!("Expected player 0 with Empty(7), got {:?}", other),
        }

        println!("✓ Successfully tested mines and high content values");
    }
}
