//! A game I created while I was still in school.
//! 
//! # Rules
//! 
//! - 2 players take turns consisting of up to one action, in case no action is possible the player has to skip this round.
//! - the players both start with one exactly one unit.
//! 
//! ```txt
//! .......o
//! ........
//! ........
//! ........
//! x.......
//! ```
//! 
//! - starting at any unit of the active player, he can take the next horizontal or vertical 2 blocks, as long as both are currently empty
//!     (`*` marks possible spots for the unit of `x` marked with `#`)
//! 
//! ```txt
//! ...x..
//! .....o
//! .**#.o
//! x..*..
//! ...*..
//! ......
//! ```
//! 
//! - or an empty horizontal or vertical block which is 3 steps away,
//!     as long as the path to the block does not contain a unit owned by this player
//!     (`*` marks possible spots for the unit of `x` marked with `#`)
//! 
//! ```txt
//! ...o...
//! .......
//! .......
//! *o.#.x.
//! x......
//! .......
//! ...*...
//! ```
//! - once both players are unable to do anything, the player with more units wins
//!     (`x` wins this game with 7 to 5)
//! 
//! ```txt
//! oxxo
//! oxxo
//! xxxo
//! ```
//! 
//! - the size and layout of the board is unspecified
use std::io::{self, Write};
use std::process;
use std::time::Duration;

use rubot::{Bot, GameBot};

mod game {
    use std::ops::Not;

    const BOARD_WIDTH: usize = 5;
    const BOARD_HEIGHT: usize = 7;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Piece {
        X,
        O,
    }

    impl Not for Piece {
        type Output = Self;

        fn not(self) -> Self {
            match self {
                Piece::X => Piece::O,
                Piece::O => Piece::X
            }
        }
    }

    pub type Tile = Option<Piece>;
    pub type Tiles = [[Tile; BOARD_HEIGHT]; BOARD_WIDTH];

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Winner {
        X,
        O,
        Tie,
    }

    #[derive(Debug, Clone)]
    pub enum MoveError {
        InvalidMove,
    }
    
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub enum Move {
        Short((usize, usize), (usize, usize)),
        Long(usize, usize),
    }

    #[derive(Debug, Clone)]
    pub struct Game {
        tiles: Tiles,
        current_piece: Piece,
        actions: Vec<Move>,
        winner: Option<Winner>,
    }

    impl Game {
        pub fn new() -> Self {
            let mut tiles: Tiles = Default::default();
            tiles[0][0] = Some(Piece::X);
            tiles[BOARD_WIDTH - 1][BOARD_HEIGHT - 1] = Some(Piece::O);
            let mut game = Self {
                tiles,
                current_piece: Piece::X,
                actions: Vec::new(),
                winner: None,
            };
            game.generate_actions(false);
            game
        }

        pub fn make_move(&mut self, mov: Move) -> Result<(), MoveError> {
            if !self.actions.contains(&mov) {
                return Err(MoveError::InvalidMove);
            }
            
            match mov {
                Move::Short((a, b), (c, d)) => {
                    self.tiles[a][b] = Some(self.current_piece);
                    self.tiles[c][d] = Some(self.current_piece);
                }
                Move::Long(a, b) => {
                    self.tiles[a][b] = Some(self.current_piece);
                }
            }
            self.current_piece = !self.current_piece;
            self.generate_actions(false);
            Ok(())
        }

        fn generate_actions(&mut self, skipped: bool) {
            self.actions.clear();
            for x in 0..BOARD_WIDTH {
                for y in 0..BOARD_HEIGHT {
                    match self.tile(x, y) {
                        Some(p) if *p == self.current_piece() => {
                            let is_inbound = |x, y| x < BOARD_WIDTH && y < BOARD_HEIGHT;

                            macro_rules! is_empty {
                                ($x:expr, $y:expr) => {
                                    self.tile($x, $y).is_none()
                                }
                            }
                            macro_rules! maybe_enemy {
                                ($x:expr, $y:expr) => {
                                    self.tile($x, $y).map_or(true, |p| p != self.current_piece())
                                }
                            }
                            macro_rules! short {
                                (($a:expr, $b:expr), ($x:expr, $y:expr)) => {
                                    if is_inbound($a, $b) && is_inbound($x, $y) 
                                    && is_empty!($a, $b) && is_empty!($x, $y) {
                                        self.actions.push(Move::Short(($a, $b), ($x, $y)));
                                    };
                                }
                            }
                            macro_rules! long {
                                (($a:expr, $b:expr), ($c:expr, $d:expr), ($x:expr, $y:expr)) => {
                                    if is_inbound($a, $b) && is_inbound($c, $d) && is_inbound($x, $y) 
                                    && maybe_enemy!($a, $b) && maybe_enemy!($c, $d) && is_empty!($x, $y) {
                                        self.actions.push(Move::Long($x, $y));
                                    };
                                }
                            }
                            short!((x.wrapping_add(1), y), (x.wrapping_add(2), y));
                            long!((x.wrapping_add(1), y), (x.wrapping_add(2), y), (x.wrapping_add(3), y));
                            short!((x.wrapping_sub(1), y), (x.wrapping_sub(2), y));
                            long!((x.wrapping_sub(1), y), (x.wrapping_sub(2), y), (x.wrapping_sub(3), y));
                            short!((x, y.wrapping_add(1)), (x, y.wrapping_add(2)));
                            long!((x, y.wrapping_add(1)), (x, y.wrapping_add(2)), (x, y.wrapping_add(3)));
                            short!((x, y.wrapping_sub(1)), (x, y.wrapping_sub(2)));
                            long!((x, y.wrapping_sub(1)), (x, y.wrapping_sub(2)), (x, y.wrapping_sub(3)));
                        }
                        _ => (),
                    }
                }
            }

            if self.actions.is_empty() {
                if !skipped {
                    self.current_piece = !self.current_piece;
                    self.generate_actions(true);
                }
                    else {
                    let stats = self.pieces();

                    use std::cmp::Ordering;
                    self.winner = Some(match stats.0.cmp(&stats.1) {
                        Ordering::Greater => Winner::X,
                        Ordering::Equal => Winner::Tie,
                        Ordering::Less => Winner::O,
                    });
                }
            }
        }

        pub fn moves(&self) -> Vec<Move> {
            self.actions.clone()
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
        
        pub fn pieces(&self) -> (usize, usize) {
            self.tiles().iter().flatten().fold((0, 0), |(x, o), t| match t {
                    Some(Piece::X) => (x + 1, o),
                    Some(Piece::O) => (x, o + 1),
                    None => (x, o)
                })
        }

        pub fn tile(&self, row: usize, col: usize) -> &Tile {
            &self.tiles[row][col]
        }

        pub fn tiles(&self) -> &Tiles {
            &self.tiles
        }
    }
}

use game::{Game, Piece, Winner, Tiles, MoveError, Move};

#[derive(Debug, Clone)]
pub struct InvalidMove(String);

fn prompt_move() -> Move {
    loop {
        print!("Enter move (e.g. A1 or B2B3): ");
        io::stdout().flush().expect("Failed to flush stdout");
        let v = read_line();
        match parse_move(&v) {
            Ok(mov) => break mov,
            Err(InvalidMove(invalid_str)) => eprintln!(
                "Invalid move: '{}'. Please try again.",
                invalid_str,
            ),
        }
    }
}

fn parse_move(input: &str) -> Result<Move, InvalidMove> {
    match input.len() {
        2 => {
            let col = match &input[0..1] {
                "A" | "a" => 0,
                "B" | "b" => 1,
                "C" | "c" => 2,
                "D" | "d" => 3,
                "E" | "e" => 4,
                "F" | "f" => 5,
                "G" | "g" => 6,
                "H" | "h" => 7,
                "I" | "i" => 8,
                "J" | "j" => 9,
                _ => return Err(InvalidMove(input.to_string())),
            };

            let row = match &input[1..2] {
                "1" => 0,
                "2" => 1,
                "3" => 2,
                "4" => 3,
                "5" => 4,
                "6" => 5,
                "7" => 6,
                "8" => 7,
                "9" => 8,
                _ => return Err(InvalidMove(input.to_string())),
            };

            Ok(Move::Long(row, col))
        },
        4 => {
            let b = match &input[0..1] {
                "A" | "a" => 0,
                "B" | "b" => 1,
                "C" | "c" => 2,
                "D" | "d" => 3,
                "E" | "e" => 4,
                "F" | "f" => 5,
                "G" | "g" => 6,
                "H" | "h" => 7,
                "I" | "i" => 8,
                "J" | "j" => 9,
                _ => return Err(InvalidMove(input.to_string())),
            };

            let a = match &input[1..2] {
                "1" => 0,
                "2" => 1,
                "3" => 2,
                "4" => 3,
                "5" => 4,
                "6" => 5,
                "7" => 6,
                "8" => 7,
                "9" => 8,
                _ => return Err(InvalidMove(input.to_string())),
            };

            let y = match &input[2..3] {
                "A" | "a" => 0,
                "B" | "b" => 1,
                "C" | "c" => 2,
                "D" | "d" => 3,
                "E" | "e" => 4,
                "F" | "f" => 5,
                "G" | "g" => 6,
                "H" | "h" => 7,
                "I" | "i" => 8,
                "J" | "j" => 9,
                _ => return Err(InvalidMove(input.to_string())),
            };

            let x = match &input[3..4] {
                "1" => 0,
                "2" => 1,
                "3" => 2,
                "4" => 3,
                "5" => 4,
                "6" => 5,
                "7" => 6,
                "8" => 7,
                "9" => 8,
                _ => return Err(InvalidMove(input.to_string())),
            };

            Ok(Move::Short((a, b), (x, y)))
        }
        _ => Err(InvalidMove(input.to_string()))
    }
}

fn read_line() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
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
            print!(" {}", match *tile {
                Some(Piece::X) => "x",
                Some(Piece::O) => "o",
                None => "\u{25A2}",
            });
        }
        println!();
    }

    println!();
}

fn main() {
    let mut game = Game::new();
    let mut opponent = Bot::new(Piece::O);
    while !game.is_finished() {
        print_tiles(game.tiles());
        match game.current_piece() {
            Piece::X => {
                println!("Current piece: x");
                let mov = prompt_move();

                match game.make_move(mov) {
                    Ok(()) => {},
                    Err(MoveError::InvalidMove) => eprintln!("The selected move was invalid"),
                }
            }
            Piece::O => {
                if let Some(mov) = opponent.select(&game, Duration::from_secs(1)) {
                    game.make_move(mov).unwrap();
                }
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

impl rubot::Game for Game {
    type Player = Piece;
    type Action = Move;
    type Actions = Vec<Move>;
    type Fitness = i32;

    /// Returns all currently possible actions and if they are executed by the given `player`
    fn actions(&self, player: &Self::Player) -> (bool, Self::Actions) {
        (*player == self.current_piece(), self.moves().clone())
    }

    /// Execute a given `action`, returning the new `fitness` for the given `player`
    fn execute(&mut self, action: &Self::Action, player: &Self::Player) -> Self::Fitness {
        match self.make_move(*action) {
            Ok(()) => (),
            Err(e) => unreachable!("Error: {:?}", e),
        }

        match player {
            Piece::X => self.pieces().0 as i32 - self.pieces().1 as i32,
            Piece::O => self.pieces().1 as i32 - self.pieces().0 as i32
        }
    }

    fn look_ahead(&self, action: &Self::Action, player: &Self::Player) -> Self::Fitness {
        let value = match player {
            Piece::X => self.pieces().0 as i32 - self.pieces().1 as i32,
            Piece::O => self.pieces().1 as i32 - self.pieces().0 as i32
        };

        let step = if let Move::Long(_, _) = action { 1 } else { 2 };
        if self.current_piece() != *player {
            value + step
        }
        else {
            value - step
        }
    }
}
