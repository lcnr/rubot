#![forbid(unsafe_code)]
//! An easily reusable game bot for deterministic games.
//!
//! It is required to implement the trait [`Game`][game] for your game to use this crate.
//!
//! # Examples
//!
//! A game where you win by selecting `10`.
//!
//! ```rust
//! use std::ops::RangeInclusive;
//!
//! #[derive(Clone)]
//! struct ChooseTen;
//!
//! impl rubot::Game for ChooseTen {
//!     /// there is only one player, you!
//!     type Player = ();
//!     type Action = u8;
//!     /// did you choose a 10?
//!     type Fitness = bool;
//!     type Actions = RangeInclusive<u8>;
//!
//!     fn actions(&self, _: Self::Player) -> (bool, Self::Actions) {
//!         (true, 1..=10)
//!     }
//!
//!     fn execute(&mut self, action: &u8, _: Self::Player) -> Self::Fitness {
//!         *action == 10
//!     }
//! }
//!
//! fn main() {
//!     use rubot::Bot;
//!     use std::time::Duration;
//!
//!     let mut bot = Bot::new(());
//!     assert_eq!(
//!         bot.select(&ChooseTen, Duration::from_secs(1)),
//!         Some(10)
//!     );
//! }
//! ```
//!
//! Please visit the [examples folder][ex] or the [`trait Game`][game] documentation
//! for more realistic examples.
//!
//! [ab]:alpha_beta/struct.Bot.html
//! [ex]:https://github.com/lcnr/rubot/tree/master/examples
//! [game]:trait.Game.html
pub mod alpha_beta;
pub mod tree;

#[allow(unused)]
#[doc(hidden)]
pub mod brute;
#[cfg(test)]
mod tests;

use std::cmp::PartialEq;
use std::ops::Drop;
use std::time::{Duration, Instant};

/// An interface required to interact with [`GameBot`s][bot].
///
/// # Examples
///
/// Implementing this trait for `21 flags`. The game has the following rules:
///
/// - at the beginning there are 21 flags.
/// - 2 players draw 1, 2 or 3 flags in alternating turns
/// - the player who removes the last flag wins
///
/// This example is really simplified and should be viewed as such.
/// For more realistic examples visit the [`/examples`][examples] folder of this project.
///
/// ```rust
/// use std::{
///     ops::{Not, RangeInclusive},         
///     time::Duration
/// };
///
/// #[derive(Clone)]
/// struct Game {
///     flags: u32,
///     active_player: Player
/// }
///
/// #[derive(Clone, Copy, PartialEq, Eq, Debug)]
/// enum Player {
///     A,
///     B
/// }
///
/// impl Not for Player {
///     type Output = Player;
///
///     fn not(self) -> Player {
///         match self {
///             Player::A => Player::B,
///             Player::B => Player::A,
///         }
///     }
/// }
///
/// impl Game {
///     fn remove_flags(&mut self, flags: u32) {
///         self.flags -= flags;
///         self.active_player = !self.active_player;
///     }
///
///     fn winner(&self) -> Player {
///         assert_eq!(self.flags, 0);
///         !self.active_player
///     }
/// }
///
/// impl rubot::Game for Game {
///     type Player = Player;
///     type Action = u32;
///     /// `true` if the player wins the game, `false` otherwise.
///     type Fitness = bool;
///     type Actions = RangeInclusive<u32>;
///     
///     fn actions(&self, player: Self::Player) -> (bool, Self::Actions) {
///         (player == self.active_player, 1..=std::cmp::min(self.flags, 3))
///     }
///     
///     fn execute(&mut self, action: &Self::Action, player: Self::Player) -> Self::Fitness {
///         (action, player, &self);
///         self.remove_flags(*action);
///         self.flags == 0 && player == self.winner()
///     }
///     
///     /// The fitness is only `true` if the game is won
///     fn is_upper_bound(&self, fitness: Self::Fitness, player: Self::Player) -> bool {
///         fitness
///     }
/// }
///
/// fn main() {
///     use rubot::{Bot, ToCompletion};
///     let mut player_a = Bot::new(Player::A);
///     let mut player_b = Bot::new(Player::B);
///     let mut game = Game { flags: 21, active_player: Player::A };
///     loop {
///         game.remove_flags(player_a.select(&game, ToCompletion).unwrap());
///         if game.flags == 0 { break }
///
///         game.remove_flags(player_b.select(&game, ToCompletion).unwrap());
///         if game.flags == 0 { break }
///     }
///     // in case both players play perfectly, the player who begins should always win
///     assert_eq!(game.winner(), Player::A, "players are not playing optimally");
/// }
/// ```
///
/// # Template
///
/// A template which can be used to implement this trait more quickly.
///
/// ```rust
/// #[derive(Clone)]
/// struct PlaceholderGame;
/// #[derive(Clone, Copy)]
/// struct PlaceholderPlayer;
/// #[derive(PartialEq)]
/// struct PlaceholderAction;
/// #[derive(Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
/// struct PlaceholderFitness;
///
/// impl rubot::Game for PlaceholderGame {
///     type Player = PlaceholderPlayer;
///     type Action = PlaceholderAction;
///     type Fitness = PlaceholderFitness;
///     type Actions = Vec<Self::Action>;
///     
///     fn actions(&self, player: Self::Player) -> (bool, Self::Actions) {
///         unimplemented!("")
///     }
///     
///     fn execute(&mut self, action: &Self::Action, player: Self::Player) -> Self::Fitness {
///         unimplemented!("");
///     }
/// }
/// ```
/// [bot]: trait.GameBot.html
/// [act]: trait.Game.html#associatedtype.Action
/// [player]: trait.Game.html#associatedtype.player
/// [examples]: https://github.com/lcnr/rubot/tree/master/examples
pub trait Game: Clone {
    /// the player type
    type Player: Copy;
    /// a executable action
    type Action: PartialEq;
    /// the fitness of a state
    type Fitness: Ord + Copy;
    /// the collection returned by [`actions`][ac]
    ///
    /// [ac]:trait.Game.html#tymethod.actions
    type Actions: IntoIterator<Item = Self::Action>;

    /// Returns all currently possible actions and if they are executed by the given `player`.
    fn actions(&self, player: Self::Player) -> (bool, Self::Actions);

    /// Execute a given `action`, returning the new `fitness` for the given `player`.
    /// The returned fitness is always from the perspective of `player`,
    /// even if the `player` is not active.
    ///
    /// A correctly implemented `GameBot` will only call this function with
    /// actions generated by [`actions`][actions].
    ///
    /// [actions]: trait.Game.html#tymethod.actions
    fn execute(&mut self, action: &Self::Action, player: Self::Player) -> Self::Fitness;

    /// Returns the fitness after `action` is executed.
    /// The returned fitness is always from the perspective of `player`,
    /// even if the `player` is not active.
    ///
    /// This function should always return the same [`Fitness`][fit] as calling [`execute`][exe].
    ///
    /// ```rust
    /// # use rubot::Game;
    /// # #[derive(Clone)]
    /// # struct GameState;
    /// # impl rubot::Game for GameState {
    /// #     type Player = ();
    /// #     type Action = ();
    /// #     type Fitness = bool;
    /// #     type Actions = Option<()>;
    /// #
    /// #     fn actions(&self, _player: Self::Player) -> (bool, Self::Actions) {
    /// #         (true, Some(()))
    /// #     }
    /// #
    /// #     fn execute(&mut self, _action: &Self::Action, _player: Self::Player) -> Self::Fitness {
    /// #         true
    /// #     }
    /// # }
    /// # let player = ();
    /// # let action = ();
    /// let mut state = GameState;
    ///
    /// let look_ahead = state.look_ahead(&action, player);
    /// let execute = state.execute(&action, player);
    ///
    /// assert_eq!(look_ahead, execute);
    /// ```
    /// [fit]: trait.Game.html#associatedtype.Fitness
    /// [exe]: trait.Game.html#tymethod.execute
    #[inline]
    fn look_ahead(&self, action: &Self::Action, player: Self::Player) -> Self::Fitness {
        self.clone().execute(action, player)
    }

    /// Returns `true` if the given `fitness` is one of the best currently possible outcomes for the given `player`.
    ///
    /// A good example is a checkmate in chess, as there does not exist a better game state than having won.
    #[inline]
    fn is_upper_bound(&self, fitness: Self::Fitness, player: Self::Player) -> bool {
        let _ = (fitness, player);
        false
    }

    /// Returns `true` if the given `fitness` is one of the worst currently possible outcomes for the given `player`.
    ///
    /// A good example is a checkmate in chess, as there does not exist a worse game state than having lost.
    #[inline]
    fn is_lower_bound(&self, fitness: Self::Fitness, player: Self::Player) -> bool {
        let _ = (fitness, player);
        false
    }
}

/// Converts a type into a [`RunCondition`][rc] used by [`Bot::select`][sel].
/// It is recommended to mostly use [`Duration`][dur].
///
/// # Examples
///
/// ```rust
/// # struct Game;
/// # struct Bot;
/// # impl Bot {
/// #   fn select<U: rubot::IntoRunCondition>(&mut self, state: &Game, condition: U) -> Option<()> {
/// #       Some(())
/// #   }
/// # }
/// use std::time::Duration;
///
/// let available_time = Duration::from_secs(2);
///
/// let game: Game = // ...
/// # Game;
/// let mut bot: Bot = // ...
/// # Bot;
/// assert!(bot.select(&game, available_time).is_some())
/// ```
/// 
/// # Implementations
/// 
/// - [`Duration`][dur]: `select` runs for the specified duration
/// - [`ToCompletion`][complete]: `select` runs until it found the perfect action
/// - [`Depth`][depth]: `select` analyses up the to given depth and returns to best action at that depth
/// - [`Instant`][instant]: `select` runs until the given `Instant` is in the past
/// - [`Logger`][logger]: takes another run condition and stores information about the last call to `select`
/// 
/// [rc]: trait.RunCondition.html
/// [dur]: https://doc.rust-lang.org/std/time/struct.Duration.html
/// [complete]: struct.ToCompletion.html
/// [depth]: struct.Depth.html
/// [instant]: https://doc.rust-lang.org/std/time/struct.Instant.html
/// [logger]: struct.Logger.html
/// [sel]: alpha_beta/struct.Bot.html#method.select
/// 
pub trait IntoRunCondition {
    type RunCondition: RunCondition;

    /// consumes `self` and returns a `RunCondition`.
    ///
    /// [rc]: trait.RunCondition.html
    fn into_run_condition(self) -> Self::RunCondition;
}

impl<T> IntoRunCondition for T
where
    T: RunCondition,
{
    type RunCondition = Self;

    fn into_run_condition(self) -> Self {
        self
    }
}

/// Can be converted into [`RunCondition`][rc] which returns `true` for the first `self.0` steps.
/// This should only be used for debugging and testing as unlike `Duration`, `ToCompletion` or `Depth`, as
/// the total amount of steps needed is not directly indicative of search depth and can change between minor versions.
///
/// [rc]: trait.RunCondition.html
#[derive(Clone, Copy, Debug)]
pub struct Steps(pub u32);

/// The [`RunCondition`][rc] created by [`Steps`][steps]`::into_run_condition`
///
/// [rc]: trait.RunCondition.html
/// [steps]: struct.Steps.html
#[doc(hidden)]
pub struct InnerSteps(u32, u32);

impl IntoRunCondition for Steps {
    type RunCondition = InnerSteps;

    fn into_run_condition(self) -> InnerSteps {
        InnerSteps(0, self.0)
    }
}

impl RunCondition for InnerSteps {
    #[inline]
    fn step(&mut self) -> bool {
        self.0 += 1;
        self.0 < self.1
    }

    #[inline]
    fn depth(&mut self, _: u32) -> bool {
        true
    }
}

/// Creates a [`RunCondition`][rc] which returns `true` until this `Duration` has passed.
///
/// [rc]: trait.RunCondition.html
impl IntoRunCondition for Duration {
    type RunCondition = Instant;

    fn into_run_condition(self) -> Instant {
        Instant::now() + self
    }
}

/// A condition which indicates if a [`Bot::select`][sel] should keep on running.
/// It is recommended to use [`Duration`][dur] for nearly all use cases.
/// 
/// A list of all already implemented `RunCondition`s can be found [here][into]
///
/// [sel]: alpha_beta/struct.Bot.html#method.select
/// [dur]: https://doc.rust-lang.org/std/time/struct.Duration.html
/// [into]: trait.IntoRunCondition.html#implementations-1
pub trait RunCondition {
    /// Called at each search step, instantly stops all calculations by returning `false`.
    fn step(&mut self) -> bool;
    /// Called after every finished search depth, instantly stops all calculations by returning `false`.
    fn depth(&mut self, depth: u32) -> bool;
}

/// Returns `true` while the `Instant` is still in the future
impl RunCondition for Instant {
    #[inline]
    fn step(&mut self) -> bool {
        Instant::now() < *self
    }

    #[inline]
    fn depth(&mut self, _: u32) -> bool {
        Instant::now() < *self
    }
}

/// A struct implementing [`RunCondition`][rc] which always returns `true`.
///
/// This means that the bot will always run until the best action was found.
///
/// # Examples
///
/// ```rust
/// # use rubot::{Bot, tree::Node, ToCompletion};
/// let tree = Node::root().with_children(&[
///     Node::new(false, 7).with_children(&[
///         Node::new(true, 4),
///         Node::new(true, 2),
///     ]),
///     Node::new(false, 5).with_children(&[
///         Node::new(true, 8),
///         Node::new(true, 9)
///     ]),
/// ]);
///
/// let mut bot = Bot::new(true);
/// assert_eq!(bot.select(&tree, ToCompletion), Some(1));
/// ```
/// [rc]: trait.RunCondition.html
#[derive(Clone, Copy, Debug)]
pub struct ToCompletion;

impl RunCondition for ToCompletion {
    #[inline]
    fn step(&mut self) -> bool {
        true
    }

    #[inline]
    fn depth(&mut self, _: u32) -> bool {
        true
    }
}

/// A struct implementing [`RunCondition`][rc] returning `false` once the current depth is bigger than `self.0`.
///
/// # Examples
///
/// ```rust
/// # use rubot::{Bot, tree::Node, Depth};
/// let tree = Node::root().with_children(&[
///     Node::new(false, 7).with_children(&[
///         Node::new(true, 4),
///         Node::new(true, 2),
///     ]),
///     Node::new(false, 5).with_children(&[
///         Node::new(true, 8),
///         Node::new(true, 9)
///     ]),
/// ]);
///
/// let mut bot = Bot::new(true);
/// assert_eq!(bot.select(&tree, Depth(0)), Some(0));
/// assert_eq!(bot.select(&tree, Depth(1)), Some(1));
/// ```
/// [rc]: trait.RunCondition.html
#[derive(Clone, Copy, Debug)]
pub struct Depth(pub u32);

impl RunCondition for Depth {
    #[inline]
    fn step(&mut self) -> bool {
        true
    }

    #[inline]
    fn depth(&mut self, depth: u32) -> bool {
        self.0 > depth
    }
}

/// A struct implementing [`IntoRunCondition`] which can be used to log a call to [`select`][sel].
/// For more details you can visit the individual methods.
///
/// # Examples
///
/// ```rust
/// # use rubot::{Bot, tree::Node, ToCompletion, Logger};
/// # use std::time::Duration;
/// let tree = Node::root().with_children(&[
///     Node::new(false, 7).with_children(&[
///         Node::new(true, 4),
///         Node::new(true, 2),
///     ]),
///     Node::new(false, 5).with_children(&[
///         Node::new(true, 8),
///         Node::new(true, 9)
///     ]),
/// ]);
///
/// let mut bot = Bot::new(true);
/// let mut logger = Logger::new(ToCompletion);
/// assert_eq!(bot.select(&tree, &mut logger), Some(1));
///
/// assert_eq!(logger.depth(), 1);
/// // the total duration of `bot.select`
/// assert!(logger.duration() < Duration::from_secs(1));
/// ```
/// [sel]: alpha_beta/struct.Bot.html#method.select
pub struct Logger<T: IntoRunCondition> {
    condition: T::RunCondition,
    steps: u32,
    depth: u32,
    completed: bool,
    duration: Duration,
}

impl<T: IntoRunCondition> Logger<T> {
    /// Creates a new `Logger` wrapping `condition`.
    pub fn new(condition: T) -> Self {
        Self {
            condition: condition.into_run_condition(),
            steps: 0,
            depth: 0,
            completed: true,
            duration: Duration::from_secs(0),
        }
    }

    /// Returns the total amount of times [`step`][step] was called during the last call to [`select`][sel].
    ///
    /// [step]: trait.RunCondition.html#tymethod.step
    /// [sel]: alpha_beta/struct.Bot.html#method.select
    pub fn steps(&self) -> u32 {
        self.steps
    }

    /// Returns the deepest completed depth of the last call to [`select`][sel].
    ///
    /// [sel]: alpha_beta/struct.Bot.html#method.select
    pub fn depth(&self) -> u32 {
        self.depth
    }

    /// Returns `true` if last call to [`select`][sel] was completed and `false` if it was
    /// cancelled by the run condition.
    ///
    /// [sel]: alpha_beta/struct.Bot.html#method.select
    pub fn completed(&self) -> bool {
        self.completed
    }

    /// Returns the total time spend during the last call to [`select`][sel].
    ///
    /// [sel]: alpha_beta/struct.Bot.html#method.select
    pub fn duration(&self) -> Duration {
        self.duration
    }

    /// consumes `self` and returns the wrapped `condition`
    pub fn into_inner(self) -> T::RunCondition {
        self.condition
    }
}

/// The [`RunCondition`][rc] created by [`Logger`][logger]`::into_run_condition`
///
/// [rc]: trait.RunCondition.html
/// [logger]: struct.Logger.html
#[doc(hidden)]
pub struct InnerLogger<'a, T: IntoRunCondition>(&'a mut Logger<T>, Instant);

impl<'a, T: IntoRunCondition> IntoRunCondition for &'a mut Logger<T> {
    type RunCondition = InnerLogger<'a, T>;

    fn into_run_condition(self) -> InnerLogger<'a, T> {
        self.steps = 0;
        self.depth = 0;
        InnerLogger(self, Instant::now())
    }
}

impl<'a, T: IntoRunCondition> RunCondition for InnerLogger<'a, T> {
    #[inline]
    fn step(&mut self) -> bool {
        self.0.steps += 1;
        if self.0.condition.step() {
            true
        } else {
            self.0.completed = false;
            false
        }
    }

    #[inline]
    fn depth(&mut self, depth: u32) -> bool {
        self.0.depth = depth;
        if self.0.condition.depth(depth) {
            true
        } else {
            self.0.completed = false;
            false
        }
    }
}

impl<'a, T: IntoRunCondition> Drop for InnerLogger<'a, T> {
    fn drop(&mut self) {
        self.0.duration = self.1.elapsed();
    }
}

pub use alpha_beta::Bot;
