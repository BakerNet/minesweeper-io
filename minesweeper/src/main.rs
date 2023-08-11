use ansi_term::Style;
use std::io;

use minesweeper::{
    board::BoardPoint,
    game::{Action, Minesweeper, PlayOutcome},
};

fn underline(input: &str) -> ansi_term::ANSIGenericString<str> {
    Style::new().underline().paint(input)
}

fn main() {
    let flags = xflags::parse_or_exit! {
        optional -i,--intermediate
        optional -e, --expert
    };
    let mut cols = 9;
    let mut rows = 9;
    let mut mines = 10;
    if flags.intermediate {
        cols = 16;
        rows = 16;
        mines = 40;
    }
    if flags.expert {
        cols = 30;
        rows = 16;
        mines = 99;
    }
    let mut game = Minesweeper::init_game(rows, cols, mines, 1).unwrap();
    while !game.is_over() {
        let curr_board = &game.player_board(0);
        let header = (0..cols)
            .map(|x| format!("|{}", x / 10))
            .collect::<String>();
        println!("{}", &format!("XX{}|", header));
        let header = (0..cols)
            .map(|x| format!("|{}", x % 10))
            .collect::<String>();
        println!("{}", underline(&format!("XX{}|", header)));
        for (r_num, row) in curr_board.iter().enumerate() {
            print!("{}", underline(&format!("{:0>2}", r_num)));
            for item in row.iter() {
                print!("{}", underline(&format!("|{:?}", item)));
            }
            print!("{}", underline("|\n"));
        }
        println!("Input action & 2 numbers `{{c|d|f}} {{row}} {{col}}` as play:");
        let mut play = String::new();

        io::stdin()
            .read_line(&mut play)
            .expect("Failed to read line");
        let play = play.trim_end().split(' ');
        if play.clone().count() != 3 {
            println!("Bad number of inputs - try again.");
            continue;
        }
        let mut play = play.into_iter();

        let action = match play.next().unwrap() {
            "c" => Action::Reveal,
            "d" => Action::RevealAdjacent,
            "f" => Action::Flag,
            _ => {
                println!("Bad action - try again");
                continue;
            }
        };
        let row = play.next().unwrap().parse();
        let Ok(row) = row else {
            println!("Invalid row - try again: {:?}", row);
            continue;
        };
        let col = play.next().unwrap().parse();
        let Ok(col) = col else {
            println!("Invalid col - try again: {:?}", col);
            continue;
        };

        let res = game.play(0, action, BoardPoint { row, col });
        if let Err(e) = res {
            println!("Invalid action - try again: {:?}", e);
            continue;
        }
        match res.unwrap() {
            PlayOutcome::Success(_) => println!("Success"),
            PlayOutcome::Failure(_) => println!("You Died"),
            PlayOutcome::Victory(_) => println!("You won!!!"),
            PlayOutcome::Flag(_) => println!("Flagged"),
        }
    }
}
