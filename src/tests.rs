use crate::{Game, GameBot};

use std::ops::Range;
use std::time::Duration;
use std::fmt::{self, Debug, Formatter};

#[derive(Clone, PartialEq, Eq)]
struct Node {
    player: bool,
    // always from the perspective of the tested player
    fitness: u32,
    children: &'static [Node]
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
    type Fitness = u32;
    type Actions= Range<usize>;

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
    const fn new(player: bool, fitness: u32) -> Self {
        const EMPTY_ARR: &[Node] = &[];
        Self {
            player,
            fitness,
            children: EMPTY_ARR
        }
    }

    const fn children(self, children: &'static [Node]) -> Self {
        Self {
            children,
            ..self
        }
    }
}

fn bots() -> Vec<Box<dyn GameBot<Node>>> {
    vec![
        Box::new(crate::Bot::new(true)),
    ]
}

const EMPTY: Node = Node::new(true, 0);

/// Who would have ever imagined that a length of 0 can cause problems.
/// I obviously did not, that's why I had to add this test.
#[test]
fn empty() {
    for mut bot in bots() {
        assert_eq!(bot.select(&EMPTY, Duration::from_secs(1)), None);
    }
}

const DEPTH_ONE: Node = Node::new(true, 0).children(&[
    Node::new(true, 0),
    Node::new(true, 2),
    Node::new(true, 1)
]);

/// Tests if the trivial case works
#[test]
fn depth_one() {
    for mut bot in bots() {
        // Some(1) to love
        assert_eq!(bot.select(&DEPTH_ONE, Duration::from_secs(1)), Some(1));
    }
}

const DIFFERENT_DEPTHS: Node = Node::new(true, 0).children(&[
    Node::new(false, 0).children(&[
        Node::new(true, 0)
    ]),
    Node::new(false, 1)
]);

/// Tests if terminating nodes get ignored in case another branch is longer
#[test]
fn different_depths() {
    for mut bot in bots() {
        // Some(1) to love
        assert_eq!(bot.select(&DIFFERENT_DEPTHS, Duration::from_secs(1)), Some(1));
    }
}

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
    ])
]);

/// This test tries to catch errors where alpha values are not removed after each depth,
/// which can cause a beta cutoff at [1][0][0], causing the returned fitness to be 4 instead of 2.
#[test]
fn alpha_reuse() {
    for mut bot in bots() {
        assert_eq!(bot.select(&ALPHA_REUSE, Duration::from_secs(1)), Some(0));
    }
}

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
        ])
    ])
]);

/// Removing a branch which seems to terminate can be dangerous in case the reason, that no deeper nodes are
/// visited, is a cutoff. This test checks this by having a beta cutoff at [2][0]. In case the branch
/// gets stored as having a fitness of 4, instead of only having a fitness of **at most** 4,
/// [2] ends up getting preferred over [1], even though the actual fitness values are 2 and 3 respectively.
/// 
/// Note: [0] is a deep branch with only bad options to prevent another bug from interfering. <3
#[test]
fn premature_termination() {
    for mut bot in bots() {
        assert_eq!(bot.select(&PREMATURE_TERMINATION, Duration::from_secs(1)), Some(1));
    }
}