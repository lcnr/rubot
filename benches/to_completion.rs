use criterion::*;

use rubot::{Bot, ToCompletion};

#[path = "chess.rs"]
mod chess;

use chess::Chess;
use shakmaty::Color;

fn criterion_benchmark(c: &mut Criterion) {
    let chess = Chess::from_fen("8/8/5KPk/8/8/8/8/8 w - -");
    let chess = Chess::from_fen("1nbqkbQB/3pp2p/4r3/2p2p2/R3P3/1P1P4/2P2PPP/1N2KBNR w - -");

    panic!(
        "best move: {:?}",
        Bot::new(Color::White).select(&chess, ToCompletion)
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
