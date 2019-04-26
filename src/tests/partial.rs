//! tests where the Bot is interrupted during computation
use super::*;
use crate::{Depth, Steps};

#[rustfmt::skip]
const FUZZ_ONE: Node = Node::root().children(&[
    Node::new(true, -1),
    Node::new(true, 65).children(&[
        Node::new(false, 0)
    ]),
    Node::new(true, 11),
]);

/// do not select [0] as it is worse than [1] and [2] at lower depths
#[test]
fn fuzz_one() {
    let selected = Bot::new(true).select(&FUZZ_ONE, Steps(2));
    assert!(
        [Some(1), Some(2)]
            .iter()
            .find(|&action| action == &selected)
            .is_some(),
        "actual: {:?}",
        selected
    );
}

const FUZZ_TWO: Node = Node::root().children(&[Node::new(true, 0), Node::new(true, 0)]);

/// `select` should always return `Some` if there is a possible action
#[test]
fn fuzz_two() {
    let selected = Bot::new(true).select(&FUZZ_TWO, Steps(0));
    assert!(selected.is_some());

    let selected = Bot::new(true).select(&FUZZ_TWO, Depth(0));
    assert!(selected.is_some());
}
