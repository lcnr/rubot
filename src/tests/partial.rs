//! tests where the Bot is interrupted during computation
use super::*;

/// do not select [0] as it is worse than [1] and [2] at lower depths
#[test]
fn fuzz_one() {
    #[rustfmt::skip]
    let fuzz_one = Node::root().children(vec![
        Node::new(true, -1),
        Node::new(true, 65).children(vec![
            Node::new(false, 0),
        ]),
        Node::new(true, 11),
    ]);
    let selected = Bot::new(true).select(&fuzz_one, Steps(2));
    assert!(
        [Some(1), Some(2)]
            .iter()
            .find(|&action| action == &selected)
            .is_some(),
        "actual: {:?}",
        selected
    );
}

/// `select` should always return `Some` if there is a possible action
#[test]
fn fuzz_two() {
    #[rustfmt::skip]
    let fuzz_two = Node::root().children(vec![
        Node::new(true, 0),
        Node::new(true, 0),
    ]);

    let selected = Bot::new(true).select(&fuzz_two, Steps(0));
    assert!(selected.is_some());

    let selected = Bot::new(true).select(&fuzz_two, Depth(0));
    assert!(selected.is_some());
}
