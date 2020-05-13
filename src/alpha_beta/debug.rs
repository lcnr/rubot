use crate::Game;

use std::fmt::{self, Debug};

use super::{BestAction, MiniMax, State, Terminated};

impl<T: Game> Debug for MiniMax<T>
where
    T::Action: Debug,
    T::Fitness: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MiniMax::Terminated(path, branch) => write!(f, "Terminated({:?}, {:?})", path, branch),
            MiniMax::Open(path, branch) => write!(f, "Open({:?}, {:?})", path, branch),
            MiniMax::DeadEnd => write!(f, "DeadEnd"),
        }
    }
}

impl<T: Game> Debug for State<T>
where
    T::Action: Debug,
    T::Fitness: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BestAction")
            .field("alpha", &self.alpha)
            .field("beta", &self.beta)
            .field("best_fitness", &self.best_fitness)
            .field("path", &self.path)
            .field("terminated", &self.terminated)
            .field("active", &self.active)
            .finish()
    }
}

impl<T: Game> Debug for BestAction<T>
where
    T::Action: Debug,
    T::Fitness: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("BestAction")
            .field("action", &self.action)
            .field("fitness", &self.fitness)
            .field("path", &self.path)
            .finish()
    }
}

impl<T: Game> Debug for Terminated<T>
where
    T::Action: Debug,
    T::Fitness: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Terminated")
            .field("best_action", &self.best_action)
            .field("partial", &self.partial)
            .finish()
    }
}
