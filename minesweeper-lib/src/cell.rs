use std::fmt::{self, Display, Formatter};

use crate::board::CompactSerialize;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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

impl Display for PlayerCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

impl CompactSerialize for PlayerCell {
    fn to_compact_byte(&self) -> u8 {
        match self {
            PlayerCell::Hidden(HiddenCell::Empty) => 0,
            PlayerCell::Hidden(HiddenCell::Mine) => 1,
            PlayerCell::Hidden(HiddenCell::Flag) => 2,
            PlayerCell::Hidden(HiddenCell::FlagMine) => 3,
            PlayerCell::Revealed(RevealedCell { player, contents }) => {
                // For revealed cells, use bit-based encoding
                // Values 4-255 are reserved for revealed cells
                // Format: Base(4) + PPPP CCCC (4 bits player, 4 bits contents)
                let base = 4u8;
                let player_bits = ((*player as u8) & 0x0F) << 4; // Upper 4 bits: player (0-15)
                let contents_code = match contents {
                    Cell::Empty(n) => (*n).min(8), // Lower 4 bits: empty count (0-8)
                    Cell::Mine => 9,               // Lower 4 bits: mine (9)
                };
                base + player_bits + contents_code
            }
        }
    }

    fn from_compact_byte(byte: u8) -> Self {
        match byte {
            0 => PlayerCell::Hidden(HiddenCell::Empty),
            1 => PlayerCell::Hidden(HiddenCell::Mine),
            2 => PlayerCell::Hidden(HiddenCell::Flag),
            3 => PlayerCell::Hidden(HiddenCell::FlagMine),
            b if b >= 4 => {
                // Revealed cell - extract using bit operations
                // Format: Base(4) + PPPP CCCC (4 bits player, 4 bits contents)
                let adjusted = b - 4;
                let player = ((adjusted >> 4) & 0x0F) as usize; // Extract upper 4 bits: player
                let contents_code = adjusted & 0x0F; // Extract lower 4 bits: contents
                let contents = if contents_code == 9 {
                    Cell::Mine
                } else {
                    Cell::Empty(contents_code)
                };
                PlayerCell::Revealed(RevealedCell { player, contents })
            }
            _ => PlayerCell::Hidden(HiddenCell::Empty), // Default fallback
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
    #[serde(rename = "p", alias = "player")]
    pub player: usize,
    #[serde(rename = "c", alias = "contents")]
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
            Self::Mine => bail!("Plant on mine not allowed"),
        }
    }

    pub fn unplant(self, num: u8) -> Result<Self> {
        match self {
            Self::Empty(_) => bail!("Unplant on empty not allowed"),
            Self::Mine => Ok(Cell::Empty(num)),
        }
    }

    pub fn is_mine(&self) -> bool {
        matches!(self, Self::Mine)
    }

    pub fn value(&self) -> Option<u8> {
        match self {
            Self::Empty(x) => Some(*x),
            Self::Mine => None,
        }
    }
}
