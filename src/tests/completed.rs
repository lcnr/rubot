//! tests where the Bot completely analyses the tree and should select the best action
use super::*;
use crate::ToCompletion;

#[rustfmt::skip]
const EMPTY: Node = Node::root();

/// Who would have ever imagined that a length of 0 can cause problems.
/// I obviously did not, that's why I had to add this test.
#[test]
fn empty() {
    assert_eq!(Bot::new(true).select(&EMPTY, ToCompletion), None);
}

#[rustfmt::skip]
const DEPTH_ONE: Node = Node::root().children(&[
        Node::new(true, 0),
        Node::new(true, 2),
        Node::new(true, 1)
    ]);

/// Tests if the trivial case works
#[test]
fn depth_one() {
    // Some(1) to love
    assert_eq!(Bot::new(true).select(&DEPTH_ONE, ToCompletion), Some(1));
}

#[rustfmt::skip]
const DIFFERENT_DEPTHS: Node = Node::root().children(&[
    Node::new(false, 0).children(&[
        Node::new(true, 0)
    ]),
    Node::new(false, 1),
]);

/// Tests if terminating nodes get ignored in case another branch is longer
#[test]
fn different_depths() {
    // Some(1) to love
    assert_eq!(
        Bot::new(true).select(&DIFFERENT_DEPTHS, ToCompletion),
        Some(1)
    );
}

#[rustfmt::skip]
const ALPHA_REUSE: Node = Node::root().children(&[
    Node::new(false, 0).children(&[
        Node::new(false, 5).children(&[
            Node::new(true, 3)
        ])
    ]),
    Node::new(false, 1).children(&[
        Node::new(false, 6).children(&[
            Node::new(true, 4),
            Node::new(true, 2)
        ])
    ]),
]);

/// This test tries to catch errors where alpha values are not removed after each depth,
/// which can cause a beta cutoff at [1][0][0], causing the returned fitness to be 4 instead of 2.
#[test]
fn alpha_reuse() {
    assert_eq!(Bot::new(true).select(&ALPHA_REUSE, ToCompletion), Some(0));
}

#[rustfmt::skip]
const PREMATURE_TERMINATION: Node = Node::root().children(&[
    Node::new(false, 0).children(&[
        Node::new(false, 0).children(&[
            Node::new(false, 0).children(&[
                Node::new(false, 0)
            ])
        ])
    ]),
    Node::new(false, 0).children(&[
        Node::new(true, 5).children(&[
            Node::new(true, 5).children(&[
                Node::new(true, 3)
            ])
        ])
    ]),
    Node::new(false, 0).children(&[
        Node::new(false, 4),
        Node::new(false, 2).children(&[
            Node::new(false, 2)
        ]),
    ]),
]);

/// Removing a branch which seems to terminate can be dangerous in case no deeper nodes are
/// visited due to a cutoff. This test checks this by having a beta cutoff at [2][0]. In case the branch
/// gets stored as having a fitness of 4, instead of only having a fitness of **at most** 4,
/// [2] ends up getting preferred over [1], even though the actual fitness values are 2 and 3 respectively.
///
/// Note: [0] is a deep branch with only bad options to prevent another bug from interfering. <3
#[test]
fn premature_termination() {
    assert_eq!(
        Bot::new(true).select(&PREMATURE_TERMINATION, ToCompletion),
        Some(1)
    );
}

#[rustfmt::skip]
const FUZZ_ONE: Node = Node::root().children(&[
    // fitness: 3
    Node::new(true, 0).children(&[
        Node::new(false, 0).children(&[
            Node::new(false, 1),
        ]),
        Node::new(true, 0),
    ]),
    // fitness: 0
    Node::new(false, 0).children(&[
        Node::new(false, 0).children(&[
            Node::new(true, 2).children(&[
                Node::new(true, 3).children(&[
                    Node::new(true, 0)
                ])
            ]),
        ]),
    ]),
    // fitness: 0
    Node::new(true, 0)
]);

/// The world is weird.
#[test]
fn fuzz_one() {
    assert_eq!(Bot::new(true).select(&FUZZ_ONE, ToCompletion), Some(0));
}

#[rustfmt::skip]
const FUZZ_TWO: Node = Node::root().children(&[
    // fitness 32
    Node::new(true, 74).children(&[
        Node::new(true, 2).children(&[
            Node::new(false, 1).children(&[
                Node::new(false, -119)
            ]),
            Node::new(true, 42)
        ]),
    ]),
    // fitness 0
    Node::new(true, 0).children(&[
        Node::new(false, 0).children(&[
            Node::new(true, -1),
            Node::new(true, 42),
        ]),
        Node::new(true, 0),
    ]),
]);

/// error due to incorrect interpretation of the exact cutoff in [1][0][1]
#[test]
fn fuzz_two() {
    assert_eq!(Bot::new(true).select(&FUZZ_TWO, ToCompletion), Some(0));
}

#[rustfmt::skip]
const FUZZ_THREE: Node = Node::root().children(&[
    // fitness 3
    Node::new(false, 1).children(&[
        Node::new(false, 2).children(&[
            Node::new(true, 10).children(&[
                Node::new(true, 3),
                Node::new(true, 10).children(&[
                    Node::new(true, 0)
                ]),
            ]),
        ]),
    ]),
    // fitness 8
    Node::new(true, 8).children(&[
        Node::new(true, 2),
        Node::new(false, 0).children(&[
            Node::new(true, 8),
            Node::new(true, 10)
        ]),
    ]),
]);

#[test]
fn fuzz_three() {
    assert_eq!(Bot::new(true).select(&FUZZ_THREE, ToCompletion), Some(1));
}
