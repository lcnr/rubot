use std::io;
use std::time::Duration;

use shakmaty::{uci::Uci, Color, Move, Position, Setup};

// reusing the chess code
#[path = "../benches/chess.rs"]
mod chess;

use chess::Chess;

fn main() {
    use rubot::Bot;
    let mut bot = Bot::new(Color::White);
    let mut game = Chess::default();
    while !game.0.is_game_over() {
        let mov = match game.0.turn() {
            Color::White => bot.select(&game, Duration::from_secs(2)).unwrap(),
            Color::Black => get_move(&game),
        };

        game.0.play_unchecked(&mov);
        println!("{:?}", game.0.board());
    }
}

fn get_move(chess: &Chess) -> Move {
    println!("Your turn, please input a move in UCI notation: ");
    loop {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");
        let len_without_newline = input.trim_end().len();
        input.truncate(len_without_newline);
        match Uci::from_ascii(input.as_bytes()) {
            Ok(uci) => match uci.to_move(&chess.0) {
                Ok(mov) => break mov,
                Err(_) => println!("Invalid move! Please try again: "),
            },
            Err(_) => println!("Invalid input, moves are expected in UCI notation: "),
        }
    }
}
