use crate::{Bot, Game, RunToCompletion};

use std::fmt::{self, Debug, Formatter};
use std::ops::Range;

#[derive(Clone, PartialEq, Eq)]
struct Node {
    player: bool,
    // always from the perspective of the tested player
    fitness: i8,
    children: &'static [Node],
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Node")
            .field("player", &self.player)
            .field("fitness", &self.fitness)
            .finish()
    }
}

impl Game for Node {
    type Player = bool;
    type Action = usize;
    type Fitness = i8;
    type Actions = Range<usize>;

    fn actions(&self, player: &Self::Player) -> (bool, Self::Actions) {
        (*player == self.player, 0..self.children.len())
    }

    fn execute(&mut self, action: &Self::Action, _: &Self::Player) -> Self::Fitness {
        *self = self.children[*action].clone();
        // fitness of the child
        self.fitness
    }

    fn look_ahead(&self, action: &Self::Action, _: &Self::Player) -> Self::Fitness {
        self.children[*action].fitness
    }
}

impl Node {
    const fn new(player: bool, fitness: i8) -> Self {
        const EMPTY_ARR: &[Node] = &[];
        Self {
            player,
            fitness,
            children: EMPTY_ARR,
        }
    }

    const fn children(self, children: &'static [Node]) -> Self {
        Self { children, ..self }
    }
}

#[rustfmt::skip]
const EMPTY: Node = Node::new(true, 0);

/// Who would have ever imagined that a length of 0 can cause problems.
/// I obviously did not, that's why I had to add this test.
#[test]
fn empty() {
    assert_eq!(Bot::new(true).select(&EMPTY, RunToCompletion), None);
}

#[rustfmt::skip]
const DEPTH_ONE: Node = Node::new(true, 0).children(&[
        Node::new(true, 0),
        Node::new(true, 2),
        Node::new(true, 1)
    ]);

/// Tests if the trivial case works
#[test]
fn depth_one() {
    // Some(1) to love
    assert_eq!(Bot::new(true).select(&DEPTH_ONE, RunToCompletion), Some(1));
}

#[rustfmt::skip]
const DIFFERENT_DEPTHS: Node = Node::new(true, 0).children(&[
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
        Bot::new(true).select(&DIFFERENT_DEPTHS, RunToCompletion),
        Some(1)
    );
}

#[rustfmt::skip]
const ALPHA_REUSE: Node = Node::new(true, 0).children(&[
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
    assert_eq!(
        Bot::new(true).select(&ALPHA_REUSE, RunToCompletion),
        Some(0)
    );
}

#[rustfmt::skip]
const PREMATURE_TERMINATION: Node = Node::new(true, 0).children(&[
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
        Bot::new(true).select(&PREMATURE_TERMINATION, RunToCompletion),
        Some(1)
    );
}

#[rustfmt::skip]
const FUZZ_ONE: Node = Node::new(true, 0).children(&[
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
    assert_eq!(Bot::new(true).select(&FUZZ_ONE, RunToCompletion), Some(0));
}

#[rustfmt::skip]
const FUZZ_TWO: Node = Node::new(true, 0).children(&[
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
    assert_eq!(Bot::new(true).select(&FUZZ_TWO, RunToCompletion), Some(0));
}
