use ansi_term::Style;
use std::io;

use minesweeper_lib::{
    board::{Board, BoardPoint},
    cell::PlayerCell,
    game::{Action, MinesweeperBuilder, MinesweeperOpts, Play, PlayOutcome},
};

fn underline(input: &str) -> ansi_term::ANSIGenericString<str> {
    Style::new().underline().paint(input)
}

fn main() {
    let flags = xflags::parse_or_exit! {
        optional -i,--intermediate
        optional -e, --expert
    };
    let opts = if flags.expert {
        MinesweeperOpts {
            cols: 30,
            rows: 16,
            num_mines: 99,
        }
    } else if flags.intermediate {
        MinesweeperOpts {
            cols: 16,
            rows: 16,
            num_mines: 40,
        }
    } else {
        MinesweeperOpts {
            rows: 9,
            cols: 9,
            num_mines: 10,
        }
    };
    let mut game = MinesweeperBuilder::new(opts).unwrap().init();
    while !game.is_over() {
        print_board(&game.player_board(0));

        let Some(play) = read_play() else {
            continue;
        };

        let res = game.play(play);
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

fn print_board(board: &Board<PlayerCell>) {
    let header = (0..board.cols()).fold(String::new(), |acc, x| acc + &format!("|{}", x / 10));
    println!("{}", &format!("XX{}|", header));
    let header = (0..board.cols()).fold(String::new(), |acc, x| acc + &format!("|{}", x % 10));
    println!("{}", underline(&format!("XX{}|", header)));
    for (r_num, row) in board.rows_iter().enumerate() {
        print!("{}", underline(&format!("{:0>2}", r_num)));
        for item in row.iter() {
            print!("{}", underline(&format!("|{}", item)));
        }
        print!("{}", underline("|\n"));
    }
}

fn read_play() -> Option<Play> {
    println!("Input action & 2 numbers `{{c|d|f}} {{row}} {{col}}` as play:");
    let mut play = String::new();

    io::stdin()
        .read_line(&mut play)
        .expect("Failed to read line");
    let play = play.trim_end().split(' ');
    if play.clone().count() != 3 {
        println!("Bad number of inputs - try again.");
        return None;
    }
    let mut play = play.into_iter();

    let action = match play.next().unwrap() {
        "c" => Action::Reveal,
        "d" => Action::RevealAdjacent,
        "f" => Action::Flag,
        _ => {
            println!("Bad action - try again");
            return None;
        }
    };
    let row = play.next().unwrap().parse();
    let Ok(row) = row else {
            println!("Invalid row - try again: {:?}", row);
            return None;
        };
    let col = play.next().unwrap().parse();
    let Ok(col) = col else {
            println!("Invalid col - try again: {:?}", col);
            return None;
        };

    Some(Play {
        player: 0,
        action,
        point: BoardPoint { row, col },
    })
}
