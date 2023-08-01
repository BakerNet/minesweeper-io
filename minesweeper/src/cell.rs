use std::fmt;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::board::BoardPoint;

#[derive(Clone, Copy)]
pub enum PlayerCell {
    Hidden,
    Flag,
    Revealed(RevealedCell),
}

impl fmt::Debug for PlayerCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hidden => write!(f, "-"),
            Self::Flag => write!(f, "F"),
            Self::Revealed(rc) => write!(
                f,
                "{}",
                if let Some(v) = rc.contents.value() {
                    format!("{v}")
                } else {
                    "X".to_string()
                }
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct RevealedCell {
    pub cell_point: BoardPoint,
    pub player: usize,
    pub contents: Cell,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CellState {
    pub revealed: bool,
    pub player: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Cell {
    Empty(u8),
    Bomb,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Empty(0)
    }
}

impl Cell {
    pub fn increment(self) -> Self {
        match self {
            Self::Empty(x) => Cell::Empty(x + 1),
            Self::Bomb => Cell::Bomb,
        }
    }

    pub fn decrement(self) -> Self {
        match self {
            Self::Empty(x) => Cell::Empty(x - 1),
            Self::Bomb => Cell::Bomb,
        }
    }

    pub fn plant(self) -> Result<Self> {
        match self {
            Self::Empty(_) => Ok(Cell::Bomb),
            Self::Bomb => bail!("Plant on bomb not allowed"),
        }
    }

    pub fn unplant(self, num: u8) -> Result<Self> {
        match self {
            Self::Empty(_) => bail!("Unplant on empty not allowed"),
            Self::Bomb => Ok(Cell::Empty(num)),
        }
    }

    pub fn is_bomb(&self) -> bool {
        matches!(self, Self::Bomb)
    }

    pub fn value(&self) -> Option<u8> {
        match self {
            Self::Empty(x) => Some(*x),
            Self::Bomb => None,
        }
    }
}
