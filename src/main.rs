use anyhow::Result;
use std::{io, num::ParseIntError};

use minesweeper::{Action, CellPoint, Minesweeper, PlayOutcome};

fn main() {
    let mut game = Minesweeper::init_game(9, 9, 10, 1).unwrap();
    while !game.is_over() {
        println!("{}", &game);
        println!("Input action & 2 numbers `{{c|d|f}} {{row}} {{col}}` as play:");
        let mut play = String::new();

        io::stdin()
            .read_line(&mut play)
            .expect("Failed to read line");
        let play = play
            .trim_end()
            .split(' ')
            .into_iter()
            .map(|x| x.parse())
            .collect::<Result<Vec<usize>, ParseIntError>>();
        let Ok(play) = play else {
            println!("Invalid input - try again: {:?}", play);
            continue;
        };
        if play.len() != 2 {
            println!("Input too long - try again.");
            continue;
        }
        println!("You played: {:?}", play);

        let res = game.play(
            0,
            Action::Click,
            CellPoint {
                row: *play.get(0).unwrap(),
                col: *play.get(1).unwrap(),
            },
        );
        if let Err(e) = res {
            println!("Invalid action - try again: {:?}", e);
            continue;
        }
        match res.unwrap() {
            PlayOutcome::Success(_) => println!("Success"),
            PlayOutcome::Failure(_) => println!("You Died"),
            PlayOutcome::Victory(_) => println!("You won!!!"),
        }
    }
}
