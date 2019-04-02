pub mod alpha_beta;
pub mod brute;
use std::time::Duration;

pub trait Game: Clone {
    type Player;
    type Action;
    type Actions: IntoIterator<Item = Self::Action>;
    type Fitness: Ord + Copy + std::fmt::Debug;

    /// Returns all currently possible actions and if they are executed by the given `player`
    fn actions(&self, player: &Self::Player) -> (bool, Self::Actions);

    /// Execute a given `action`, returning the new `fitness` for the given `player`
    ///
    /// A correctly implemented `GameBot` will only use actions generated by `fn actions()`,
    /// meaning that wrong actions do not have to be correctly handled,
    /// as long as they do not cause [undefined behavior].
    ///
    /// [undefined behavior]:https://doc.rust-lang.org/beta/reference/behavior-considered-undefined.html
    fn execute(&mut self, action: &Self::Action, player: &Self::Player) -> Self::Fitness;

    /// Returns the fitness after `action` is executed
    fn look_ahead(&self, action: &Self::Action, player: &Self::Player) -> Self::Fitness {
        self.clone().execute(action, player)
    }
}

pub trait GameBot<T: Game> {
    /// Returns a chosen action based on the given game state.
    ///
    /// In case no `Action` is possible or the bot is not the currently active player, this functions returns `None`.
    /// This methodd runs for a duration which is smaller or slightly larger than `duration`.
    fn select(&mut self, state: &T, duration: Duration) -> Option<T::Action>;
}

/// The currently recommended game bot, the actual implementation used is bound to change during development
pub use alpha_beta::Bot as Bot;