//! A tree implementation used in examples and tests.

use crate::Game;
use std::convert::TryInto;
use std::fmt::{self, Debug, Formatter};
use std::num::Wrapping;
use std::ops::Range;

/// A tree node, implements [`Game`][game].
///
/// # Examples
///
/// ```rust
/// use rubot::{Bot, ToCompletion, tree::Node};
///
/// # #[rustfmt::skip]
/// let tree = Node::root().with_children(&[
///     Node::new(false, 4),
///     Node::new(false, 7).with_children(&[
///         Node::new(true, 5),
///         Node::new(true, 3),
///     ])
/// ]);
///
/// let mut bot = Bot::new(true);
/// assert_eq!(bot.select(&tree, ToCompletion), Some(0));
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

impl Game for Node {
    type Player = bool;
    type Action = usize;
    type Fitness = i8;
    type Actions = Range<usize>;

    const UPPER_LIMIT: Option<i8> = Some(std::i8::MAX);
    const LOWER_LIMIT: Option<i8> = Some(std::i8::MIN);

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
    /// Creates a root node, this is equal to `Node::new(true, 0)`.
    pub fn root() -> Self {
        Self::new(true, 0)
    }

    /// Creates a new node with no children.
    pub fn new(player: bool, fitness: i8) -> Self {
        Self {
            player,
            fitness,
            children: Vec::new(),
        }
    }

    /// Generates a tree from `bytes`, the total amount of tree nodess, excluding the root,
    /// is currently `cmp::max(, bytes.len() - 4)`.
    ///
    /// The exact algorithm is not specified, so while the output is deterministic, it is not stable between versions
    /// and changing it will not be a breaking change.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        match bytes[0..4].try_into() {
            Ok(seed) => {
                let mut seed = u32::from_be_bytes(seed);
                if seed == 0 {
                    seed = 0xBAD_5EED;
                }

                struct XorShiftRng {
                    x: Wrapping<u32>,
                    y: Wrapping<u32>,
                    z: Wrapping<u32>,
                    w: Wrapping<u32>,
                }

                impl XorShiftRng {
                    fn next_u32(&mut self) -> u32 {
                        let x = self.x;
                        let t = x ^ (x << 11);
                        self.x = self.y;
                        self.y = self.z;
                        self.z = self.w;
                        let w = self.w;
                        self.w = w ^ (w >> 19) ^ (t ^ (t >> 8));
                        self.w.0
                    }
                }

                let mut rng = XorShiftRng {
                    x: Wrapping(seed),
                    y: Wrapping(seed),
                    z: Wrapping(seed),
                    w: Wrapping(seed),
                };

                let mut root = Node::new(true, 0);
                for &i in bytes[4..].iter() {
                    let mut pos = Some(&mut root);
                    let mut next =
                        rng.next_u32() as usize % (pos.as_ref().unwrap().children.len() + 1);
                    while next != pos.as_ref().unwrap().children.len() {
                        pos.take().map(|node| {
                            pos = Some(&mut node.children[next]);
                        });
                        next = rng.next_u32() as usize % (pos.as_ref().unwrap().children.len() + 1);
                    }

                    pos.unwrap()
                        .children
                        .push(Node::new(rng.next_u32() % 2 == 0, i as i8));
                }

                root
            }
            Err(_) => Self::root(),
        }
    }

    /// Sets the children of `self` to `children`.
    /// Previous children are forgotten.
    pub fn with_children(mut self, children: &[Node]) -> Self {
        self.children.clear();
        self.children.extend_from_slice(children);
        self
    }

    // Add a `child` node to `self`
    pub fn push_child(&mut self, child: Node) {
        self.children.push(child);
    }

    /// Returns how many children this node possesses
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Returns whether or not this node is a leaf, meaning that
    /// it does not have any children
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}
