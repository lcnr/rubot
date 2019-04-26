//! tests where the Bot is interrupted during computation
use super::*;
use crate::Steps;

#[rustfmt::skip]
const FUZZ_ONE: Node = Node::new(true, 0).children(&[
    Node::new(true, -1),
    Node::new(true, 65).children(&[
        Node::new(false, 0)
    ]),
    Node::new(true, 11),
]);

// do not select [0] as it is worse than [1] and [2] at lower depths
#[test]
fn fuzz_one() {
    let selected = Bot::new(true).select(&FUZZ_ONE, Steps(2));
    assert!(
        [Some(1), Some(2)]
            .iter()
            .find(|&action| action == &selected)
            .is_some(),
        "Actual: {:?}",
        selected
    );
}
