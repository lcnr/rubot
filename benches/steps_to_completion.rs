use rubot::{Bot, ToCompletion, Logger};

#[path = "chess.rs"]
mod chess;

use chess::Chess;
use shakmaty::Setup;

fn count_steps(name: &str, fen: &str) {
    let chess = Chess::from_fen(fen);
    let mut bot = Bot::new(chess.0.turn());
    let mut logger = Logger::new(ToCompletion);
    bot.select(&chess, &mut logger);
    println!("{:060} {:10}", name, logger.steps());
}

/// http://wtharvey.com/m8n2.txt
fn mate_in_two() {
    count_steps("Gustav Neumann vs Carl Mayet, Berlin, 1866", "5rkr/pp2Rp2/1b1p1Pb1/3P2Q1/2n3P1/2p5/P4P2/4R1K1 w - - 1 0");
    count_steps("Joseph Blackburne vs Martin, England, 1876", "1r1kr3/Nbppn1pp/1b6/8/6Q1/3B1P2/Pq3P1P/3RR1K1 w - - 1 0");
    count_steps("Wilfried Paulsen vs Adolf Anderssen, Frankfurt, 1878", "5rk1/1p1q2bp/p2pN1p1/2pP2Bn/2P3P1/1P6/P4QKP/5R2 w - - 1 0");
    count_steps("Joseph Blackburne vs Smith, Simul, 1882", "r1nk3r/2b2ppp/p3b3/3NN3/Q2P3q/B2B4/P4PPP/4R1K1 w - - 1 0");
    println!();
}

/// http://wtharvey.com/m8n3.txt
fn mate_in_three() {
    count_steps("Daniel Harrwitz vs Bernhard Horwitz, London, 1846", "3q1r1k/2p4p/1p1pBrp1/p2Pp3/2PnP3/5PP1/PP1Q2K1/5R1R w - - 1 0");
    count_steps("Bernhard Horwitz vs Howard Staunton, London, 1846", "6k1/ppp2ppp/8/2n2K1P/2P2P1P/2Bpr3/PP4r1/4RR2 b - - 0 1");
    count_steps("J Schulten vs Bernhard Horwitz, London, 1846", "rn3rk1/p5pp/2p5/3Ppb2/2q5/1Q6/PPPB2PP/R3K1NR b - - 0 1");
    count_steps("NN vs Henry Bird, England, 1850", "N1bk4/pp1p1Qpp/8/2b5/3n3q/8/PPP2RPP/RNB1rBK1 b - - 0 1");
    println!();
}

/// http://wtharvey.com/m8n4.txt
fn mate_in_four() {
    count_steps("Paul Morphy vs NN, New Orleans (blind, simul), 1858 ", "r1b3kr/3pR1p1/ppq4p/5P2/4Q3/B7/P5PP/5RK1 w - - 1 0");
    count_steps("Ignac Kolisch vs Luigi Centurini, Geneva, 1859", "2k4r/1r1q2pp/QBp2p2/1p6/8/8/P4PPP/2R3K1 w - - 1 0");
    count_steps("Paul Morphy vs Samuel Boden, London, 1859", "2r1r3/p3P1k1/1p1pR1Pp/n2q1P2/8/2p4P/P4Q2/1B3RK1 w - - 1 0");
    count_steps("Jules De Riviere vs Paul Journoud, Paris, 1860", "r1bk3r/pppq1ppp/5n2/4N1N1/2Bp4/Bn6/P4PPP/4R1K1 w - - 1 0");
    println!();
}

fn main() {
    println!("{:065} steps", "game");
    mate_in_two();
    mate_in_three();
    mate_in_four();
}