//! These benchmarks use moments from games by https://lichess.org/@/rubot_simple/all

use rubot::{Bot, Depth, Logger};

#[path = "chess.rs"]
mod chess;

use chess::Chess;
use shakmaty::Setup;

fn count_steps(name: &str, fen: &str, depth: u32) {
    let chess = Chess::from_fen(fen);
    let mut bot = Bot::new(chess.0.turn());
    let mut logger = Logger::new(Depth(depth));
    bot.select(&chess, &mut logger);
    println!("{:060} {:10}", name, logger.steps());
    assert!(!logger.completed());
}

fn depth_three() {
    count_steps("rubot_simple vs handschaf 10+0, 01.05.2019", "6k1/2ppqp1p/1p2p1p1/1b6/8/r3PPPQ/5K1P/6NR b - - 3 34", 3);
    count_steps("rubot_simple vs gobok 10+0, 01.05.2019", "2kr3r/1pp4p/p4b2/P4Rp1/1PP1p2P/8/8/2R1K3 b - - 0 32", 3);
    count_steps("tespitedilemedi vs rubot_simple 3+2, 20.04.2019", "7k/4Q3/1p2P1pr/1B1P1p1p/p2P4/P7/1P3PPP/5K2 b - - 0 34", 3);
    count_steps("tespitedilemedi vs rubot_simple 3+0, 20.04.2019", "1nq3n1/1kp1p2p/1p2p3/7P/r4p2/PN3P2/1Q3PB1/1R1R2K1 b - - 1 30", 3);
    println!();
}

fn depth_four() {
    count_steps("rubot_simple vs CgaDeaimann 3+0, 20.04.2019", "3r2k1/1p2p2p/4p1p1/1p6/5P2/1P1P3P/P3KPP1/q1N4R b - - 1 27", 4);
    count_steps("jianz vs rubot_simple 1+0, 20.04.2019", "rn2kbnr/p1pp1ppp/1p2pq2/3P4/2b1P3/1P6/P1P2PPP/RNBQK1NR w KQkq - 1 6", 4);
    count_steps("rubot_simple vs NbChessMaster 5+0, 20.04.2019", "r2q1rk1/pp1n1p1p/2p3p1/3pbb2/1P2n3/P1P1Q3/1B2PPPP/RN2KBNR b KQ - 5 13", 4);
    count_steps("rubot_simple vs JoeDalton01 1+0, 20.04.2019", "q4rk1/1ppbbpp1/8/3p4/rn4Pp/P2P2B1/2P1P1PP/RNQK1BNR b - - 0 17", 4);
    println!();
}

fn depth_five() {
    count_steps("rubot_simple vs Oleg20 10+0, 20.04.2019", "8/7k/3pp1p1/6RP/7P/P1p4K/8/8 b - - 0 52", 5);
    count_steps("handschaf vs rubot_simple 5+0, 19.04.2019", "r3k1r1/1pp2p1p/p2bP3/1P6/2P1Q3/3BP3/qB4PP/1R1R2K1 b q - 0 23", 5);
    count_steps("gambit2009 vs rubot_simple 5+0, 25.04.2019", "rn1qkbnr/1bpp3p/1p2pp2/4P1N1/p1BP1B2/P1N5/1PP2PPP/R2Q1RK1 b - - 0 11", 5);
    count_steps("gambit2009 vs rubot_simple 5+0, 24.04.2019", "4r2r/1b4pk/2p1Pp2/1pBp1PP1/p7/P1PB4/2P5/2KRR3 b - - 0 29", 5);
    println!();
}

fn depth_six() {
    count_steps("MsBlueberries vs rubot_simple 3+1, 24.04.2019", "r1k3nr/3q2pp/1pp2p2/8/p2b1B2/P6P/1PP1Q1PN/3R3K w - - 1 27", 6);
    count_steps("rubot_simple vs NuclearKnight 0+5, 24.04.2019", "rnbqkbnr/pppp1ppp/8/4p3/8/1P6/P1PPPPPP/RNBQKBNR w KQkq - 0 2", 6);
    count_steps("NuclearKnight vs rubot_simple 3+2, 24.04.2019", "rnq1k1nr/1bpp1ppp/1p2p3/8/Bb1PP3/2N2N2/PPP2PPP/R1BQ1RK1 w kq - 1 9", 6);
    count_steps("handschaf vs rubot_simple 5+0, 30.04.2019", "1r2kbnr/1q1p1ppp/4p3/p3N3/PpRP4/1Q2P3/1B3PPP/1R4K1 w k - 1 23", 6);
    println!();
}

fn main() {
    println!("{:065} steps", "game");
    depth_three();
    depth_four();
    depth_five();
    depth_six();
}