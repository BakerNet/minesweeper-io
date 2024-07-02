use std::fmt;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlayerCell {
    #[serde(rename = "r", alias = "Revealed")]
    Revealed(RevealedCell),
    #[serde(untagged)]
    Hidden(HiddenCell),
}

impl Default for PlayerCell {
    fn default() -> Self {
        Self::Hidden(HiddenCell::Empty)
    }
}

impl fmt::Debug for PlayerCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Hidden(hc) => match hc {
                HiddenCell::Empty => write!(f, "-"),
                HiddenCell::Mine => write!(f, "*"),
                HiddenCell::Flag => write!(f, "f"),
                HiddenCell::FlagMine => write!(f, "F"),
            },
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

impl PlayerCell {
    pub fn add_flag(self) -> Self {
        match self {
            Self::Revealed(_) => self,
            Self::Hidden(hc) => match hc {
                HiddenCell::Empty => Self::Hidden(HiddenCell::Flag),
                HiddenCell::Mine => Self::Hidden(HiddenCell::FlagMine),
                _ => self,
            },
        }
    }

    pub fn remove_flag(self) -> Self {
        match self {
            Self::Revealed(_) => self,
            Self::Hidden(hc) => match hc {
                HiddenCell::Flag => Self::Hidden(HiddenCell::Empty),
                HiddenCell::FlagMine => Self::Hidden(HiddenCell::Mine),
                _ => self,
            },
        }
    }

    pub fn into_hidden(self) -> Self {
        match self {
            Self::Hidden(_) => self,
            Self::Revealed(rc) if matches!(rc.contents, Cell::Mine) => {
                Self::Hidden(HiddenCell::Mine)
            }
            Self::Revealed(_) => Self::Hidden(HiddenCell::Empty),
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum HiddenCell {
    #[serde(rename = "e", alias = "Hidden")]
    Empty,
    #[serde(rename = "m", alias = "Bomb", alias = "Mine")]
    Mine, // post-game only
    #[serde(rename = "f", alias = "Flag")]
    Flag,
    #[serde(rename = "fm", alias = "FlagMine")]
    FlagMine, // post-game only
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RevealedCell {
    pub player: usize,
    pub contents: Cell,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CellState {
    pub revealed: bool,
    pub player: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Eq, PartialOrd)]
pub enum Cell {
    #[serde(rename = "e", alias = "Empty")]
    Empty(u8),
    #[serde(rename = "m", alias = "Bomb", alias = "Mine")]
    Mine,
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
            Self::Mine => Cell::Mine,
        }
    }

    pub fn decrement(self) -> Self {
        match self {
            Self::Empty(x) => Cell::Empty(x - 1),
            Self::Mine => Cell::Mine,
        }
    }

    pub fn plant(self) -> Result<Self> {
        match self {
            Self::Empty(_) => Ok(Cell::Mine),
            Self::Mine => bail!("Plant on bomb not allowed"),
        }
    }

    pub fn unplant(self, num: u8) -> Result<Self> {
        match self {
            Self::Empty(_) => bail!("Unplant on empty not allowed"),
            Self::Mine => Ok(Cell::Empty(num)),
        }
    }

    pub fn is_bomb(&self) -> bool {
        matches!(self, Self::Mine)
    }

    pub fn value(&self) -> Option<u8> {
        match self {
            Self::Empty(x) => Some(*x),
            Self::Mine => None,
        }
    }
}
