use anyhow::{bail, Result};

use crate::{
    board::Board,
    cell::{HiddenCell, PlayerCell},
    game::{Play, PlayOutcome},
};

#[derive(Default, Clone, Copy)]
pub struct SimplePlayer {
    score: usize,
    dead: bool,
    victory_click: bool,
}

pub struct MinesweeperReplay {
    current_play: Option<Play>,
    current_board: Board<PlayerCell>,
    current_players: Vec<SimplePlayer>,
    current_flags: usize,
    current_revealed_mines: usize,
    log: Vec<(Play, PlayOutcome)>,
    current_pos: usize,
}

impl MinesweeperReplay {
    pub fn new(
        starting_board: Board<PlayerCell>,
        log: Vec<(Play, PlayOutcome)>,
        players: usize,
    ) -> Self {
        Self {
            current_board: starting_board,
            current_play: None,
            current_players: vec![SimplePlayer::default(); players],
            current_flags: 0,
            current_revealed_mines: 0,
            log,
            current_pos: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.log.len() + 1
    }

    pub fn is_empty(&self) -> bool {
        false
    }

    pub fn current_pos(&self) -> usize {
        self.current_pos
    }

    pub fn current_play(&self) -> Option<Play> {
        self.current_play
    }

    pub fn current_board(&self) -> &Board<PlayerCell> {
        &self.current_board
    }

    pub fn current_players(&self) -> &Vec<SimplePlayer> {
        &self.current_players
    }

    pub fn current_flags_and_revealed_mines(&self) -> usize {
        self.current_flags + self.current_revealed_mines
    }

    pub fn advance(&mut self) -> Result<()> {
        if self.current_pos == self.len() - 1 {
            bail!("Called next on end")
        }
        let play = &self.log[self.current_pos];
        self.current_play = Some(play.0);
        match &play.1 {
            PlayOutcome::Success(results) => results.iter().for_each(|rc| {
                self.current_players[rc.1.player].score += 1;
                self.current_board[rc.0] = PlayerCell::Revealed(rc.1);
            }),
            PlayOutcome::Failure(rc) => {
                self.current_players[rc.1.player].dead = true;
                self.current_revealed_mines += 1;
                self.current_board[rc.0] = PlayerCell::Revealed(rc.1);
            }
            PlayOutcome::Victory(results) => {
                self.current_players[results[0].1.player].victory_click = true;
                results.iter().for_each(|rc| {
                    self.current_players[rc.1.player].score += 1;
                    self.current_board[rc.0] = PlayerCell::Revealed(rc.1);
                });
            }
            PlayOutcome::Flag(res) => {
                if matches!(res.1, PlayerCell::Hidden(HiddenCell::Flag)) {
                    self.current_flags += 1;
                    self.current_board[res.0] = self.current_board[res.0].add_flag()
                } else {
                    self.current_flags += 1;
                    self.current_board[res.0] = self.current_board[res.0].remove_flag()
                }
            }
        };
        self.current_pos += 1;
        Ok(())
    }

    pub fn rewind(&mut self) -> Result<()> {
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
                self.current_players[rc.1.player].score -= 1;
                self.current_board[rc.0] = PlayerCell::Hidden(HiddenCell::Empty);
            }),
            PlayOutcome::Failure(rc) => {
                self.current_players[rc.1.player].dead = false;
                self.current_revealed_mines -= 1;
                self.current_board[rc.0] = PlayerCell::Hidden(HiddenCell::Mine);
            }
            PlayOutcome::Victory(results) => {
                self.current_players[results[0].1.player].victory_click = false;
                results.iter().for_each(|rc| {
                    self.current_players[rc.1.player].score -= 1;
                    self.current_board[rc.0] = PlayerCell::Hidden(HiddenCell::Empty);
                });
            }
            PlayOutcome::Flag(res) => {
                if matches!(res.1, PlayerCell::Hidden(HiddenCell::Flag)) {
                    self.current_flags -= 1;
                    self.current_board[res.0] = self.current_board[res.0].remove_flag()
                } else {
                    self.current_flags += 1;
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
        while pos < self.current_pos {
            let _ = self.rewind();
        }
        while pos > self.current_pos {
            let _ = self.advance();
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        board::BoardPoint,
        cell::{Cell, RevealedCell},
        game::Action,
    };

    const MINES: [BoardPoint; 4] = [
        BoardPoint { row: 0, col: 3 },
        BoardPoint { row: 3, col: 0 },
        BoardPoint { row: 3, col: 2 },
        BoardPoint { row: 3, col: 3 },
    ];
    const PLAY_1_RES: [(BoardPoint, RevealedCell); 9] = [
        (
            BoardPoint { row: 0, col: 0 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(0),
            },
        ),
        (
            BoardPoint { row: 0, col: 1 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(0),
            },
        ),
        (
            BoardPoint { row: 0, col: 2 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(1),
            },
        ),
        (
            BoardPoint { row: 1, col: 0 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(0),
            },
        ),
        (
            BoardPoint { row: 1, col: 1 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(0),
            },
        ),
        (
            BoardPoint { row: 1, col: 2 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(1),
            },
        ),
        (
            BoardPoint { row: 2, col: 0 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(1),
            },
        ),
        (
            BoardPoint { row: 2, col: 1 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(2),
            },
        ),
        (
            BoardPoint { row: 2, col: 2 },
            RevealedCell {
                player: 0,
                contents: Cell::Empty(2),
            },
        ),
    ];
    const PLAY_2_RES: (BoardPoint, PlayerCell) = (
        BoardPoint { row: 3, col: 2 },
        PlayerCell::Hidden(HiddenCell::Flag),
    );
    const PLAY_3_RES: (BoardPoint, RevealedCell) = (
        BoardPoint { row: 2, col: 3 },
        RevealedCell {
            player: 0,
            contents: Cell::Empty(2),
        },
    );
    const PLAY_4_RES: (BoardPoint, RevealedCell) = (
        BoardPoint { row: 3, col: 3 },
        RevealedCell {
            player: 0,
            contents: Cell::Mine,
        },
    );

    #[test]
    fn name() {
        let mut expected_starting_board = Board::new(4, 4, PlayerCell::Hidden(HiddenCell::Empty));
        MINES.iter().for_each(|point| {
            expected_starting_board[*point] = PlayerCell::Hidden(HiddenCell::Mine);
        });
        let expected_starting_board = expected_starting_board;

        let mut expected_next_board = expected_starting_board.clone();
        // res of first play
        PLAY_1_RES.iter().for_each(|(point, rc)| {
            expected_next_board[*point] = PlayerCell::Revealed(*rc);
        });
        let expected_board_1 = expected_next_board.clone();
        // res of second play
        expected_next_board[PLAY_2_RES.0] = PlayerCell::Hidden(HiddenCell::FlagMine);
        let expected_board_2 = expected_next_board.clone();
        // res of third play
        expected_next_board[PLAY_3_RES.0] = PlayerCell::Revealed(PLAY_3_RES.1);
        let expected_board_3 = expected_next_board.clone();
        // res of final play
        expected_next_board[PLAY_4_RES.0] = PlayerCell::Revealed(PLAY_4_RES.1);
        let expected_final_board = expected_next_board.clone();

        drop(expected_next_board);

        let mut replay = MinesweeperReplay::new(
            expected_starting_board.clone(),
            Vec::from([
                (
                    Play {
                        player: 0,
                        action: Action::Reveal,
                        point: BoardPoint { row: 2, col: 2 },
                    },
                    PlayOutcome::Success(Vec::from(PLAY_1_RES)),
                ),
                (
                    Play {
                        player: 0,
                        action: Action::Flag,
                        point: BoardPoint { row: 3, col: 2 },
                    },
                    PlayOutcome::Flag(PLAY_2_RES),
                ),
                (
                    Play {
                        player: 0,
                        action: Action::Reveal,
                        point: BoardPoint { row: 2, col: 3 },
                    },
                    PlayOutcome::Success(Vec::from([PLAY_3_RES])),
                ),
                (
                    Play {
                        player: 0,
                        action: Action::Reveal,
                        point: BoardPoint { row: 3, col: 3 },
                    },
                    PlayOutcome::Failure(PLAY_4_RES),
                ),
            ]),
            2,
        );

        // test defaults
        assert_eq!(replay.current_players.len(), 2);
        assert_eq!(
            replay
                .current_players
                .iter()
                .map(|p| p.score)
                .sum::<usize>(),
            0
        );
        assert_eq!(replay.current_flags, 0);
        assert_eq!(replay.current_revealed_mines, 0);
        assert_eq!(replay.len(), 5);

        // test advance
        assert!(matches!(replay.advance(), Ok(())));
        assert_eq!(replay.current_board(), &expected_board_1);
        assert!(matches!(replay.advance(), Ok(())));
        assert_eq!(replay.current_board(), &expected_board_2);
        assert!(matches!(replay.advance(), Ok(())));
        assert_eq!(replay.current_board(), &expected_board_3);
        assert!(matches!(replay.advance(), Ok(())));
        assert_eq!(replay.current_board(), &expected_final_board);

        // should error on advance at end
        assert!(replay.advance().is_err());

        // test rewind
        assert!(matches!(replay.rewind(), Ok(())));
        assert_eq!(replay.current_board(), &expected_board_3);
        assert!(matches!(replay.rewind(), Ok(())));
        assert_eq!(replay.current_board(), &expected_board_2);
        assert!(matches!(replay.rewind(), Ok(())));
        assert_eq!(replay.current_board(), &expected_board_1);
        assert!(matches!(replay.rewind(), Ok(())));
        assert_eq!(replay.current_board(), &expected_starting_board);

        // should error on rewind at beginning
        assert!(replay.rewind().is_err());

        // try to_pos (auto advance/rewind)
        assert!(matches!(replay.to_pos(2), Ok(())));
        assert_eq!(replay.current_board(), &expected_board_2);
        assert!(matches!(replay.to_pos(4), Ok(())));
        assert_eq!(replay.current_board(), &expected_final_board);
        assert!(matches!(replay.to_pos(1), Ok(())));
        assert_eq!(replay.current_board(), &expected_board_1);

        assert!(replay.to_pos(5).is_err());
    }
}
