use crate::Game;

use std::fmt::{self, Debug};

use super::{Action, Branch, Ctxt, MiniMax, State};

impl<T: Game> Debug for Action<T>
where
    T::Action: Debug,
    T::Fitness: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Action")
            .field("fitness", &self.fitness)
            .field("path", &self.fitness)
            .finish()
    }
}

impl<'a, T: Game> Debug for Ctxt<'a, T>
where
    T: Debug,
    T::Action: Debug,
    T::Fitness: Debug,
    T::Player: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Ctxt")
            .field("state", &self.state)
            .field("player", &self.player)
            .field("best", &self.best)
            .field("unfinished", &self.unfinished)
            .field("terminated", &self.terminated)
            .field("partially_terminated", &self.partially_terminated)
            .finish()
    }
}

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
        f.debug_struct("State")
            .field("alpha", &self.alpha)
            .field("beta", &self.beta)
            .field("best_fitness", &self.best_fitness)
            .field("path", &self.path)
            .field("terminated", &self.terminated)
            .field("active", &self.active)
            .finish()
    }
}

impl<T: Game> Debug for Branch<T>
where
    T::Action: Debug,
    T::Fitness: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Branch::Worse(fitness) => write!(f, "Worse({:?})", fitness),
            Branch::Better(fitness) => write!(f, "Better({:?})", fitness),
            Branch::Equal(fitness) => write!(f, "Equal({:?})", fitness),
        }
    }
}
