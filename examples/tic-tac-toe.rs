//! This tic-tac-toe implementation is copied from `https://github.com/sunjay/tic-tac-toe`.
//!
//! His implementation serves as a guide/introduction to rust,
//! so there are a lot of helpful comments which I removed for this version.
//!
//! Check it out in case you are interested.
//!
//! His code is licensed under the [Mozilla Public License 2.0](https://github.com/sunjay/tic-tac-toe/blob/master/LICENSE)
//!
//! # Adding `rubot` support
//!
//! All relevant changes were made in lines 274 to 311 as well as adding a bot opponent in `main`.
use std::io::{self, Write};
use std::process;
use std::time::Duration;

mod game {
    const BOARD_SIZE: usize = 3;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Piece {
        X,
        O,
    }

    impl Piece {
        pub fn other(self) -> Piece {
            match self {
                Piece::X => Piece::O,
                Piece::O => Piece::X,
            }
        }
    }

    pub type Tile = Option<Piece>;
    pub type Tiles = [[Tile; BOARD_SIZE]; BOARD_SIZE];

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Winner {
        X,
        O,
        Tie,
    }

    #[derive(Debug, Clone)]
    pub enum MoveError {
        GameAlreadyOver,
        InvalidPosition {
            row: usize,
            col: usize,
        },
        TileNotEmpty {
            other_piece: Piece,
            row: usize,
            col: usize,
        },
    }

    #[derive(Debug, Clone)]
    pub struct Game {
        tiles: Tiles,
        current_piece: Piece,
        winner: Option<Winner>,
    }

    impl Game {
        pub fn new() -> Self {
            Self {
                tiles: Default::default(),
                current_piece: Piece::X,
                winner: None,
            }
        }

        pub fn make_move(&mut self, row: usize, col: usize) -> Result<(), MoveError> {
            if self.is_finished() {
                return Err(MoveError::GameAlreadyOver);
            } else if row >= self.tiles.len() || col >= self.tiles[0].len() {
                return Err(MoveError::InvalidPosition { row, col });
            } else if let Some(other_piece) = self.tiles[row][col] {
                return Err(MoveError::TileNotEmpty {
                    other_piece,
                    row,
                    col,
                });
            }

            self.tiles[row][col] = Some(self.current_piece);
            self.current_piece = self.current_piece.other();
            self.update_winner(row, col);
            Ok(())
        }

        fn update_winner(&mut self, row: usize, col: usize) {
            let rows = self.tiles.len();
            let cols = self.tiles[0].len();

            let tiles_row = self.tiles[row];

            let tiles_col = [self.tiles[0][col], self.tiles[1][col], self.tiles[2][col]];

            assert!(
                rows == 3 && cols == 3,
                "This code was written with the assumption that there are three rows and columns"
            );
            let tiles_diagonal_1 = if row == col {
                [self.tiles[0][0], self.tiles[1][1], self.tiles[2][2]]
            } else {
                [None, None, None]
            };

            let tiles_diagonal_2 = if (rows - row - 1) == col {
                [self.tiles[0][2], self.tiles[1][1], self.tiles[2][0]]
            } else {
                [None, None, None]
            };

            fn check_winner(row: &[Tile]) -> Option<Winner> {
                if row[0] == row[1] && row[1] == row[2] {
                    match row[0] {
                        Some(Piece::X) => Some(Winner::X),
                        Some(Piece::O) => Some(Winner::O),
                        None => None,
                    }
                } else {
                    None
                }
            }
            self.winner = self
                .winner
                .or_else(|| check_winner(&tiles_row))
                .or_else(|| check_winner(&tiles_col))
                .or_else(|| check_winner(&tiles_diagonal_1))
                .or_else(|| check_winner(&tiles_diagonal_2));

            self.winner = self.winner.or_else(|| {
                if self
                    .tiles
                    .iter()
                    .all(|row| row.iter().all(|tile| tile.is_some()))
                {
                    Some(Winner::Tie)
                } else {
                    None
                }
            });
        }

        pub fn is_finished(&self) -> bool {
            self.winner.is_some()
        }

        pub fn winner(&self) -> Option<Winner> {
            self.winner
        }

        pub fn current_piece(&self) -> Piece {
            self.current_piece
        }

        pub fn tiles(&self) -> &Tiles {
            &self.tiles
        }
    }
}

use game::{Game, MoveError, Piece, Tiles, Winner};

#[derive(Debug, Clone)]
pub struct InvalidMove(String);

fn prompt_move() -> (usize, usize) {
    loop {
        print!("Enter move (e.g. A1): ");
        io::stdout().flush().expect("Failed to flush stdout");
        let line = read_line();
        match parse_move(&line) {
            Ok((row, col)) => break (row, col),
            Err(InvalidMove(invalid_str)) => {
                eprintln!("Invalid move: '{}'. Please try again.", invalid_str,)
            }
        }
    }
}

fn parse_move(input: &str) -> Result<(usize, usize), InvalidMove> {
    if input.len() != 2 {
        return Err(InvalidMove(input.to_string()));
    }

    let col = match &input[0..1] {
        "A" | "a" => 0,
        "B" | "b" => 1,
        "C" | "c" => 2,
        invalid => return Err(InvalidMove(invalid.to_string())),
    };

    let row = match &input[1..2] {
        "1" => 0,
        "2" => 1,
        "3" => 2,
        _ => return Err(InvalidMove(input.to_string())),
    };

    Ok((row, col))
}

fn read_line() -> String {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Failed to read input");
    if input.is_empty() {
        println!();
        process::exit(0);
    }
    let len_without_newline = input.trim_end().len();
    input.truncate(len_without_newline);
    input
}

fn print_tiles(tiles: &Tiles) {
    print!("  ");
    for j in 0..tiles[0].len() as u8 {
        print!(" {}", (b'A' + j) as char);
    }
    println!();

    for (i, row) in tiles.iter().enumerate() {
        print!(" {}", i + 1);
        for tile in row {
            print!(
                " {}",
                match *tile {
                    Some(Piece::X) => "x",
                    Some(Piece::O) => "o",
                    None => "\u{25A2}",
                }
            );
        }
        println!();
    }

    println!();
}

fn main() {
    use rubot::Bot;

    let mut game = Game::new();
    let mut opponent = Bot::new(Piece::O);
    while !game.is_finished() {
        print_tiles(game.tiles());
        match game.current_piece() {
            Piece::X => {
                println!("Current piece: x");
                let (row, col) = prompt_move();

                match game.make_move(row, col) {
                    Ok(()) => {}
                    Err(MoveError::GameAlreadyOver) => {
                        unreachable!("Game was already over when it should not have been")
                    }
                    Err(MoveError::InvalidPosition { row, col }) => unreachable!(
                        "Should not be able to enter an invalid move, but still got ({}, {})",
                        row, col
                    ),
                    Err(MoveError::TileNotEmpty {
                        other_piece,
                        row,
                        col,
                    }) => eprintln!(
                        "The tile at position {}{} already has piece {} in it!",
                        row + 1,
                        (b'A' + col as u8) as char,
                        match other_piece {
                            Piece::X => "x",
                            Piece::O => "o",
                        },
                    ),
                }
            }
            Piece::O => {
                let action = opponent.select(&game, Duration::from_secs(1)).unwrap();
                game.make_move(action.0, action.1).unwrap();
            }
        }
    }
    print_tiles(game.tiles());
    match game.winner().expect("finished game should have winner") {
        Winner::X => println!("x wins!"),
        Winner::O => println!("o wins!"),
        Winner::Tie => println!("Tie!"),
    }
}

// <----------------------------------------------------------------->
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Action(usize, usize);

#[derive(Copy, Clone, PartialOrd, Ord, PartialEq, Eq)]
pub enum Fitness {
    Loss,
    /// the game is still ongoing or a tie
    Even,
    Win,
}

impl rubot::Game for Game {
    type Player = Piece;
    type Action = Action;
    type Actions = Vec<Action>;
    type Fitness = Fitness;

    fn actions(&self, player: Self::Player) -> (bool, Self::Actions) {
        let mut actions = Vec::new();
        if !self.is_finished() {
            for x in 0..3 {
                for y in 0..3 {
                    if let None = self.tiles()[x][y] {
                        actions.push(Action(x, y));
                    }
                }
            }
        }
        (player == self.current_piece(), actions)
    }

    fn execute(&mut self, action: &Self::Action, player: Self::Player) -> Self::Fitness {
        match self.make_move(action.0, action.1) {
            Ok(()) => (),
            Err(e) => unreachable!("Error: {:?}", e),
        }

        match self.winner() {
            None | Some(Winner::Tie) => Fitness::Even,
            Some(Winner::O) => {
                if player == Piece::O {
                    Fitness::Win
                } else {
                    Fitness::Loss
                }
            }
            Some(Winner::X) => {
                if player == Piece::X {
                    Fitness::Win
                } else {
                    Fitness::Loss
                }
            }
        }
    }

    fn is_upper_bound(&self, fitness: Self::Fitness, _: Self::Player) -> bool {
        fitness == Fitness::Win
    }

    fn is_lower_bound(&self, fitness: Self::Fitness, _: Self::Player) -> bool {
        fitness == Fitness::Loss
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rubot::{Bot, Logger, ToCompletion};

    #[test]
    fn first_pos() {
        let mut game = Game::new();
        game.make_move(0, 0).unwrap();

        let mut opponent = Bot::new(Piece::O);
        let mut logger = Logger::new(ToCompletion);
        assert_eq!(opponent.select(&game, &mut logger).unwrap(), Action(1, 1));
        assert!(logger.duration() < Duration::from_secs(1));
    }
}
