use crate::{Game, GameBot};

use std::ops::Range;
use std::time::Duration;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Node {
    player: bool,
    // always from the perspective of the tested player
    fitness: u32,
    children: &'static [Node]
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
        Box::new(crate::alpha_beta::Bot::new(true)),
    ]
}

const EMPTY: Node = Node::new(true, 0);

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

#[test]
fn depth_one() {
    for mut bot in bots() {
        // Some(1) to love
        assert_eq!(bot.select(&DEPTH_ONE, Duration::from_secs(1)), Some(1));
    }
}

const DEPTH_TWO: Node = Node::new(true, 0).children(&[
    Node::new(false, 0).children(&[
        Node::new(true, 0)
    ]),
    Node::new(false, 1)
]);

#[test]
fn depth_two() {
    for mut bot in bots() {
        // Some(1) to love
        assert_eq!(bot.select(&DEPTH_TWO, Duration::from_secs(1)), Some(1));
    }
}
