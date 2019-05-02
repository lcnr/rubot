use criterion::*;

use rubot::{Bot, ToCompletion};

#[path = "chess.rs"]
mod chess;

use chess::Chess;
use shakmaty::Setup;

fn bench_fen(c: &mut Criterion, name: &str, fen: &str) {
    let chess = Chess::from_fen(fen);
    let mut bot = Bot::new(chess.0.turn());
    c.bench_function(name, move |b| b.iter(|| bot.select(&chess, ToCompletion)));
}

/// http://wtharvey.com/m8n2.txt
fn mate_in_two(c: &mut Criterion) {
    bench_fen(
        c,
        "Henry Buckle vs NN, London, 1840",
        "r2qkb1r/pp2nppp/3p4/2pNN1B1/2BnP3/3P4/PPP2PPP/R2bK2R w KQkq - 1 0",
    );
    bench_fen(
        c,
        "Louis Paulsen vs Blachy, New York, 1857",
        "1rb4r/pkPp3p/1b1P3n/1Q6/N3Pp2/8/P1P3PP/7K w - - 1 0",
    );
    bench_fen(
        c,
        "Paul Morphy vs Duke Isouard, Paris, 1858",
        "4kb1r/p2n1ppp/4q3/4p1B1/4P3/1Q6/PPP2PPP/2KR4 w k - 1 0",
    );
    bench_fen(
        c,
        "Johannes Zukertort vs Adolf Anderssen, Breslau, 1865",
        "r1b2k1r/ppp1bppp/8/1B1Q4/5q2/2P5/PPP2PPP/R3R1K1 w - - 1 0",
    );
}

/// http://wtharvey.com/m8n3.txt
fn mate_in_three(c: &mut Criterion) {
    bench_fen(
        c,
        "Madame de Remusat vs Napoleon I, Paris, 1802",
        "r1b1kb1r/pppp1ppp/5q2/4n3/3KP3/2N3PN/PPP4P/R1BQ1B1R b kq - 0 1",
    );
    bench_fen(
        c,
        "William Evans vs Alexander MacDonnell, London, 1826",
        "r3k2r/ppp2Npp/1b5n/4p2b/2B1P2q/BQP2P2/P5PP/RN5K w kq - 1 0",
    );
    bench_fen(
        c,
        "William Evans vs Alexander MacDonnell, London, 1829",
        "r1b3kr/ppp1Bp1p/1b6/n2P4/2p3q1/2Q2N2/P4PPP/RN2R1K1 w - - 1 0",
    );
    bench_fen(
        c,
        "H Popert vs John Cochrane, London, 1841",
        "r2n1rk1/1ppb2pp/1p1p4/3Ppq1n/2B3P1/2P4P/PP1N1P1K/R2Q1RN1 b - - 0 1",
    );
}

criterion_group!(benches, mate_in_two, mate_in_three);
criterion_main!(benches);
