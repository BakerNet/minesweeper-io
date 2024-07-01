use anyhow::{bail, Result};

use crate::{
    board::Board,
    cell::{HiddenCell, PlayerCell},
    game::{Play, PlayOutcome},
};

pub struct MinesweeperReplay {
    current_play: Option<Play>,
    current_board: Board<PlayerCell>,
    log: Vec<(Play, PlayOutcome)>,
    current_pos: usize,
}

impl MinesweeperReplay {
    pub fn new(starting_board: Board<PlayerCell>, log: Vec<(Play, PlayOutcome)>) -> Self {
        Self {
            current_board: starting_board,
            current_play: None,
            log,
            current_pos: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.log.len() + 1
    }

    pub fn current_pos(&self) -> usize {
        self.current_pos
    }

    pub fn current_play(&self) -> Option<Play> {
        self.current_play
    }

    pub fn current_board(&self) -> Vec<Vec<PlayerCell>> {
        (&self.current_board).into()
    }

    pub fn next(&mut self) -> Result<()> {
        if self.current_pos == self.len() - 1 {
            bail!("Called next on end")
        }
        let play = &self.log[self.current_pos];
        self.current_play = Some(play.0);
        match &play.1 {
            PlayOutcome::Success(results) => results.iter().for_each(|rc| {
                self.current_board[rc.cell_point] = PlayerCell::Revealed(rc.clone());
            }),
            PlayOutcome::Failure(rc) => {
                self.current_board[rc.cell_point] = PlayerCell::Revealed(rc.clone());
            }
            PlayOutcome::Victory(results) => results.iter().for_each(|rc| {
                self.current_board[rc.cell_point] = PlayerCell::Revealed(rc.clone());
            }),
            PlayOutcome::Flag(res) => {
                if matches!(res.1, PlayerCell::Hidden(HiddenCell::Flag)) {
                    self.current_board[res.0] = self.current_board[res.0].add_flag()
                } else {
                    self.current_board[res.0] = self.current_board[res.0].remove_flag()
                }
            }
        };
        self.current_pos += 1;
        Ok(())
    }

    pub fn prev(&mut self) -> Result<()> {
        if self.current_pos == 0 {
            bail!("Called prev on start")
        }
        self.current_pos -= 1;
        let play_to_undo = &self.log[self.current_pos];
        self.current_play = if self.current_pos == 0 {
            None
        } else {
            Some(self.log[self.current_pos - 1].0)
        };
        match &play_to_undo.1 {
            PlayOutcome::Success(results) => results.iter().for_each(|rc| {
                self.current_board[rc.cell_point] = PlayerCell::Hidden(HiddenCell::Empty);
            }),
            PlayOutcome::Failure(rc) => {
                self.current_board[rc.cell_point] = PlayerCell::Hidden(HiddenCell::Mine);
            }
            PlayOutcome::Victory(results) => results.iter().for_each(|rc| {
                self.current_board[rc.cell_point] = PlayerCell::Hidden(HiddenCell::Empty);
            }),
            PlayOutcome::Flag(res) => {
                if matches!(res.1, PlayerCell::Hidden(HiddenCell::Flag)) {
                    self.current_board[res.0] = self.current_board[res.0].remove_flag()
                } else {
                    self.current_board[res.0] = self.current_board[res.0].add_flag()
                }
            }
        };
        Ok(())
    }

    pub fn to_pos(&mut self, pos: usize) -> Result<()> {
        if pos >= self.len() {
            bail!(
                "Called to_pos with pos out of bounds (max {}): {}",
                self.len() - 1,
                pos
            )
        }
        while pos > self.current_pos {
            let _ = self.prev();
        }
        while pos > self.current_pos {
            let _ = self.next();
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn name() {
        todo!();
    }
}
