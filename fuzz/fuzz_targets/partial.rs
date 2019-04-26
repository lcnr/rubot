//! runs `fn select` with a `RunCondition`.
//!
//! The returned action should either be the best action of the last completed depth
//! or have a fitness at the interrupted depth which is better than the fitness of the
//! previously best action.
#![no_main]

#[macro_use]
extern crate libfuzzer_sys;
extern crate rubot;

use std::convert::TryInto;
use std::num::Wrapping;
use std::ops::Range;

use rubot::{brute::Brute, Bot, Depth, Game, Logger, Steps, ToCompletion};
use std::fmt::{self, Debug, Formatter};

struct XorShiftRng {
    x: Wrapping<u32>,
    y: Wrapping<u32>,
    z: Wrapping<u32>,
    w: Wrapping<u32>,
}

impl XorShiftRng {
    #[inline]
    fn next_u32(&mut self) -> u32 {
        let x = self.x;
        let t = x ^ (x << 11);
        self.x = self.y;
        self.y = self.z;
        self.z = self.w;
        let w_ = self.w;
        self.w = w_ ^ (w_ >> 19) ^ (t ^ (t >> 8));
        self.w.0
    }

    #[inline]
    fn from_seed(mut seed: u32) -> Self {
        if seed == 0 {
            seed = 0xBAD_5EED;
        }

        XorShiftRng {
            x: Wrapping(seed),
            y: Wrapping(seed),
            z: Wrapping(seed),
            w: Wrapping(seed),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Node {
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
            .field("children", &self.children)
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
    fn new(player: bool, fitness: i8) -> Self {
        Self {
            player,
            fitness,
            children: Vec::new(),
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut root = Node::new(true, 0);
        let mut rng = XorShiftRng::from_seed(u32::from_be_bytes(bytes[0..4].try_into().unwrap()));

        for &i in bytes[4..].iter() {
            let mut pos = Some(&mut root);
            let mut next = rng.next_u32() as usize % (pos.as_ref().unwrap().children.len() + 1);
            while next != pos.as_ref().unwrap().children.len() {
                pos.take().map(|node| {
                    pos = Some(&mut node.children[next]);
                });
                next = rng.next_u32() as usize % (pos.as_ref().unwrap().children.len() + 1);
            }

            pos.unwrap()
                .children
                .push(Node::new(i & 1 == 0, (i ^ 1) as i8));
        }

        root
    }
}

fuzz_target!(|data: &[u8]| {
    if data.len() >= 4 {
        let node = Node::from_bytes(data);

        let (max_depth, max_steps) = {
            let mut logger = Logger::new(ToCompletion);
            Bot::new(true).select(&node, &mut logger);
            (logger.depth(), logger.steps())
        };

        for i in 0..max_steps {
            let mut logger = Logger::new(Steps(i));
            let selected = Bot::new(true).select(&node, &mut logger);
            if !Brute::new(true)
                .allowed_actions(&node, logger.depth())
                .into_iter()
                .find(|a| *a == selected)
                .is_some()
            {
                println!(
                    "Error with node: {:?}. Expected: {:?}, Actual: {:?}, Steps: {}",
                    node,
                    Brute::new(true).allowed_actions(&node, logger.depth()),
                    selected,
                    i
                );
                panic!();
            }
        }

        for i in 0..max_depth {
            let mut logger = Logger::new(Depth(i));
            let selected = Bot::new(true).select(&node, &mut logger);
            if !Brute::new(true).check_if_best(&node, selected.as_ref(), i) {
                println!(
                    "Error with node: {:?}. Expected: {:?}, Actual: {:?}, Depth: {}",
                    node,
                    Brute::new(true).allowed_actions(&node, logger.depth()),
                    selected,
                    i
                );
                panic!();
            }
        }
    }
});
