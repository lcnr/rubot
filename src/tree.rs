//! A tree implementation used in examples and tests.

use crate::prelude::*;
use std::fmt::{self, Debug, Formatter};
use std::ops::Range;

/// A tree node, implements [`Game`][game].
///
/// As `Node`s require their children to be `'static`
/// it is recommended to only use them in constants.
///
/// # Examples
///
/// ```rust
/// use rubot::{Bot, ToCompletion, tree::Node};
///
/// # #[rustfmt::skip]
/// const TREE: Node = Node::root().children(vec![
///     Node::new(false, 4),
///     Node::new(false, 7).children(vec![
///         Node::new(true, 5),
///         Node::new(true, 3),
///     ])
/// ]);
///
/// let mut bot = Bot::new(true);
/// assert_eq!(bot.select(&TREE, ToCompletion), Some(0));
/// ```
/// [game]: ../trait.Game.html
#[derive(Clone, PartialEq, Eq)]
pub struct Node {
    player: bool,
    // always from the perspective of the tested player
    fitness: i8,
    children: Vec<Node>,
}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Node")
            .field("player", &self.player)
            .field("fitness", &self.fitness)
            .finish()
    }
}

impl crate::Game for Node {
    type Player = bool;
    type Action = usize;
    type Fitness = i8;
    type Actions = Range<usize>;

    fn actions(&self, player: &Self::Player) -> (bool, Self::Actions) {
        (*player == self.player, 0..self.children.len())
    }

    fn execute(self, action: &Self::Action, _: &Self::Player) -> StepResult<Self> {
        let child = self.children[*action].clone();
        let fitness = child.fitness;
        if child.children.is_empty() {
            StepResult::Terminated(fitness)
        } else {
            StepResult::Open(child, fitness)
        }
    }
}

impl Node {
    /// creates a root node, this is equal to `Node::new(true, 0)`.
    pub fn root() -> Self {
        Self::new(true, 0)
    }

    /// creates a new node with no children.
    pub fn new(player: bool, fitness: i8) -> Self {
        Self {
            player,
            fitness,
            children: Vec::new(),
        }
    }

    /// adds children to `self`.
    pub fn children(self, children: Vec<Node>) -> Self {
        Self { children, ..self }
    }
}
