//! tests where the Bot is interrupted during computation
use super::*;

/// `select` should always return `Some` if there is a possible action
#[test]
fn no_steps() {
    #[rustfmt::skip]
    let no_steps = Node::root().push_children(&[
        Node::new(true, 0),
        Node::new(true, 0),
    ]);

    let selected = Bot::new(true).select(&no_steps, Steps(0));
    assert!(selected.is_some());

    let selected = Bot::new(true).select(&no_steps, Depth(0));
    assert!(selected.is_some());
}

/// do not select [0] as it is worse than [1] and [2] at lower depths
#[test]
fn fuzz_one() {
    #[rustfmt::skip]
    let fuzz_one = Node::root().push_children(&[
        Node::new(true, -1),
        Node::new(true, 65).push_children(&[
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
