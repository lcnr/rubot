//! A tree implementation used in examples and tests.

use crate::Game;
use std::fmt::{self, Debug, Formatter};
use std::ops::Range;

/// A tree node, implements [`Game`][game].
///
/// As `Node`s require its children to be `'static`
/// it is recommended to only use them in constants.
///
/// # Examples
///
/// ```rust
/// use rubot::{Bot, ToCompletion, tree::Node};
///
/// # #[rustfmt::skip]
/// const TREE: Node = Node::root().children(&[
///     Node::new(false, 4),
///     Node::new(false, 7).children(&[
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
    /// creates a root node, this is equal to `Node::new(true, 0)`.
    pub const fn root() -> Self {
        Self::new(true, 0)
    }

    /// creates a new node with no children.
    pub const fn new(player: bool, fitness: i8) -> Self {
        const EMPTY_ARR: &[Node] = &[];
        Self {
            player,
            fitness,
            children: EMPTY_ARR,
        }
    }

    /// adds children to `self`.
    pub const fn children(self, children: &'static [Node]) -> Self {
        Self { children, ..self }
    }
}
