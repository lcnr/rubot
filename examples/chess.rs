use std::time::Duration;
use std::io;

use shakmaty::{Move, MoveList, Color, Position, Setup, Role, uci::Uci};

/// this example requires a newtype due to orphan rules, as both shakmaty::Chess and rubot::Game
/// are from outside of this example.
#[derive(Debug, Clone, Default)]
struct Chess(shakmaty::Chess);

impl rubot::Game for Chess {
    type Player = Color;
    type Action = Move;
    type Actions = MoveList;
    type Fitness = i32;

    /// Returns all currently possible actions and if they are executed by the given `player`
    fn actions(&self, player: &Self::Player) -> (bool, Self::Actions) {
        (*player == self.0.turn(), self.0.legals())
    }

    /// Execute a given `action`, returning the new `fitness` for the given `player`
    fn execute(&mut self, action: &Self::Action, player: &Self::Player) -> Self::Fitness {
        self.0.play_unchecked(action);

        let mut fitness = 0;
        for (_square, piece) in self.0.board().pieces() {
            // values based on https://medium.freecodecamp.org/simple-chess-ai-step-by-step-1d55a9266977
            let value = match piece.role {
                Role::Pawn => 10,
                Role::Knight => 30,
                Role::Bishop => 30,
                Role::Rook => 50,
                Role::Queen => 90,
                Role::King => 900,
            };
            
            if piece.color == *player {
                fitness += value;
            }
            else {
                fitness -= value;
            }
        }
        fitness
    }
}

fn main() {
    use rubot::{GameBot, Bot};
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
        io::stdin().read_line(&mut input).expect("Failed to read input");
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