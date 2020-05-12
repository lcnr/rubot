//! tests where the Bot completely analyses the tree and should select the best action
use super::*;

/// Who would have ever imagined that a length of 0 can cause problems.
/// I obviously did not, that's why I had to add this test.
#[test]
fn empty() {
    #[rustfmt::skip]
    let empty = Node::root();

    assert_eq!(Bot::new(true).select(&empty, ToCompletion), None);
}

/// Tests if the trivial case works
#[test]
fn depth_one() {
    #[rustfmt::skip]
    let depth_one = Node::root().with_children(&[
        Node::new(true, 0),
        Node::new(true, 2),
        Node::new(true, 1),
    ]);

    // Some(1) to love
    assert_eq!(Bot::new(true).select(&depth_one, ToCompletion), Some(1));
}

/// Tests if terminating nodes get ignored in case another branch is longer
#[test]
fn different_depths() {
    #[rustfmt::skip]
    let different_depths = Node::root().with_children(&[
        Node::new(false, 0).with_children(&[
            Node::new(true, 0),
        ]),
        Node::new(false, 1),
    ]);

    // Some(1) to love
    assert_eq!(
        Bot::new(true).select(&different_depths, ToCompletion),
        Some(1)
    );
}

/// This test tries to catch errors where alpha values are not removed after each depth,
/// which can cause a beta cutoff at [1][0][0], causing the returned fitness to be 4 instead of 2.
#[test]
fn alpha_reuse() {
    #[rustfmt::skip]
    let alpha_reuse = Node::root().with_children(&[
        Node::new(false, 0).with_children(&[
            Node::new(false, 5).with_children(&[
                Node::new(true, 3),
            ]),
        ]),
        Node::new(false, 1).with_children(&[
            Node::new(false, 6).with_children(&[
                Node::new(true, 4),
                Node::new(true, 2),
            ]),
        ]),
    ]);

    assert_eq!(Bot::new(true).select(&alpha_reuse, ToCompletion), Some(0));
}

/// Removing a branch which seems to terminate can be dangerous in case no deeper nodes are
/// visited due to a cutoff. This test checks this by having a beta cutoff at [2][0]. In case the branch
/// gets stored as having a fitness of 4, instead of only having a fitness of **at most** 4,
/// [2] ends up getting preferred over [1], even though the actual fitness values are 2 and 3 respectively.
///
/// Note: [0] is a deep branch with only bad options to prevent another bug from interfering. <3
#[test]
fn premature_termination() {
    #[rustfmt::skip]
    let premature_termination = Node::root().with_children(&[
        Node::new(false, 0).with_children(&[
            Node::new(false, 0).with_children(&[
                Node::new(false, 0).with_children(&[
                    Node::new(false, 0),
                ]),
            ]),
        ]),
        Node::new(false, 0).with_children(&[
            Node::new(true, 5).with_children(&[
                Node::new(true, 5).with_children(&[
                    Node::new(true, 3),
                ]),
            ]),
        ]),
        Node::new(false, 0).with_children(&[
            Node::new(false, 4),
            Node::new(false, 2).with_children(&[
                Node::new(false, 2),
            ]),
        ]),
    ]);

    assert_eq!(
        Bot::new(true).select(&premature_termination, ToCompletion),
        Some(1)
    );
}

/// The world is weird.
#[test]
fn fuzz_one() {
    #[rustfmt::skip]
    let fuzz_one = Node::root().with_children(&[
        // fitness: 3
        Node::new(true, 0).with_children(&[
            Node::new(false, 0).with_children(&[
                Node::new(false, 1),
            ]),
            Node::new(true, 0),
        ]),
        // fitness: 0
        Node::new(false, 0).with_children(&[
            Node::new(false, 0).with_children(&[
                Node::new(true, 2).with_children(&[
                    Node::new(true, 3).with_children(&[
                        Node::new(true, 0),
                    ]),
                ]),
            ]),
        ]),
        // fitness: 0
        Node::new(true, 0),
    ]);

    assert_eq!(Bot::new(true).select(&fuzz_one, ToCompletion), Some(0));
}

/// error due to incorrect interpretation of the cutoff
/// in [1][0][1]
#[test]
fn fuzz_two() {
    #[rustfmt::skip]
    let fuzz_two = Node::root().with_children(&[
        // fitness 32
        Node::new(true, 74).with_children(&[
            Node::new(true, 2).with_children(&[
                Node::new(false, 1).with_children(&[
                    Node::new(false, -119),
                ]),
                Node::new(true, 42),
            ]),
        ]),
        // fitness 0
        Node::new(true, 0).with_children(&[
            Node::new(false, 0).with_children(&[
                Node::new(true, -1),
                Node::new(true, 42),
            ]),
            Node::new(true, 0),
        ]),
    ]);

    assert_eq!(Bot::new(true).select(&fuzz_two, ToCompletion), Some(0));
}

#[test]
fn fuzz_three() {
    #[rustfmt::skip]
    let fuzz_three = Node::root().with_children(&[
        // fitness 3
        Node::new(false, 1).with_children(&[
            Node::new(false, 2).with_children(&[
                Node::new(true, 10).with_children(&[
                    Node::new(true, 3),
                    Node::new(true, 10).with_children(&[
                        Node::new(true, 0),
                    ]),
                ]),
            ]),
        ]),
        // fitness 8
        Node::new(true, 8).with_children(&[
            Node::new(true, 2),
            Node::new(false, 0).with_children(&[
                Node::new(true, 8),
                Node::new(true, 10),
            ]),
        ]),
    ]);

    assert_eq!(Bot::new(true).select(&fuzz_three, ToCompletion), Some(1));
}

#[test]
fn fuzz_four() {
    #[rustfmt::skip]
    let fuzz_four = Node::root().with_children(&[
        Node::new(true, 0).with_children(&[
            Node::new(true, 32).with_children(&[
                Node::new(false, 127),
            ]),
        ]),
        Node::new(false, 0).with_children(&[
            Node::new(false, 0),
        ])
    ]);

    assert_eq!(Bot::new(true).select(&fuzz_four, ToCompletion), Some(0));
}

/// Tests for a bug which caused [0] to always return Terminated(Worse(1)), as [0][1][1]
/// causes an alpha beta cutoff, returning Worse(1). This replaced Equal(1)
/// because I accidentally wrote `old_fitness <= new_fitness` instead of
/// `old_fitness < new_fitness`.
#[test]
fn subtree_cutoff() {
    let subtree_cutoff = Node::root().with_children(&[
        Node::new(true, -63).with_children(&[
            // Branch::Equal(one)
            Node::new(true, 1),
            Node::new(false, -2).with_children(&[
                Node::new(false, 10).with_children(&[Node::new(true, -63)]),
                Node::new(true, 1),
            ]),
        ]),
        Node::new(false, -2),
    ]);

    assert_eq!(
        Bot::new(true).select(&subtree_cutoff, ToCompletion),
        Some(0)
    );
}
