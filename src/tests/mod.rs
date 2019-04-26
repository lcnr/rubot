use crate::{Bot, Game};

use std::fmt::{self, Debug, Formatter};
use std::ops::Range;

mod brute;
mod completed;
mod partial;

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
