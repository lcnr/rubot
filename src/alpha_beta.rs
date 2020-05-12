//! A deterministic game bot using alpha beta pruning.

use crate::{Game, IntoRunCondition, RunCondition};

use std::cmp;
use std::fmt::{self, Debug};
use std::mem;

/// A game bot which analyses its moves using [alpha beta pruning][ab_wiki] with [iterative deepening][id]. In case [`select`][sel] terminates
/// before `condition` returned true, the result is always the best possible move. While this bot caches some data
/// during computation, it does not require a lot of memory and will not store anything between different [`select`][sel] calls.
///
/// This bot requires [`Game`][game] to be implemented for your game.
///
/// # Examples
///
/// ```rust
/// use rubot::{Bot, ToCompletion, tree::Node};
/// use std::time::Duration;
///
/// let tree = Node::root().with_children(&[
///     Node::new(false, 7).with_children(&[
///         Node::new(true, 4),
///         Node::new(true, 2),
///     ]),
///     Node::new(false, 5).with_children(&[
///         Node::new(true, 8),
///         Node::new(true, 9)
///     ]),
///     Node::new(false, 6),
/// ]);
///
/// // Create a new bot for the currently active player.
/// let mut bot = Bot::new(true);
///
/// // Find the best possible action.
/// let best = bot.select(&tree, ToCompletion);
/// // Search for at most 2 seconds and return the best answer found.
/// // As 2 seconds are more than enough for this simple tree, this will
/// // return the best possible action without spending this much time.
/// let limited = bot.select(&tree, Duration::from_secs(2));
///
/// assert_eq!(best, Some(1));
/// assert_eq!(limited, Some(1));
/// ```
/// Please visit [`select`][sel] for a simple example.
///
/// [id]:https://en.wikipedia.org/wiki/Iterative_deepening_depth-first_search
/// [ab_wiki]:https://en.wikipedia.org/wiki/Alpha%E2%80%93beta_pruning
/// [sel]: struct.Bot.html#method.select
/// [game]: ../trait.Game.html
pub struct Bot<T: Game> {
    player: T::Player,
}

impl<T: Game> Bot<T> {
    /// Creates a new `Bot` for the given `player`.
    pub fn new(player: T::Player) -> Self {
        Self { player }
    }

    /// Returns a chosen action based on the given game state.
    ///
    /// Returns  `None` if no `Action` is possible or the bot is currently not the active player.
    ///
    /// This method runs until either the best possible action was found
    /// or one of `RunCondition::depth` and `RunCondition::step` returned `false`.
    pub fn select<U: IntoRunCondition>(&mut self, state: &T, condition: U) -> Option<T::Action> {
        let mut condition = condition.into_run_condition();

        let (active, actions) = state.actions(self.player);
        if !active {
            return None;
        }

        let mut actions = actions.into_iter().collect::<Vec<_>>();
        if actions.len() < 2 {
            return actions.pop();
        }
        actions.sort_by_cached_key(|a| state.look_ahead(&a, self.player));

        let mut terminated = Terminated::default();
        let mut best_action: Option<BestAction<T>> = None;
        for depth in 0.. {
            if !condition.depth(depth) {
                return current_best(terminated, best_action).or_else(|| actions.pop());
            }

            let prev_actions = mem::replace(&mut actions, Vec::new());

            if let Some(BestAction {
                path,
                action,
                fitness,
            }) = best_action.take()
            {
                let alpha = terminated.best_fitness();

                match self.rate_action(state, action, path, alpha, depth, &mut condition) {
                    RateAction::Cancelled(action) => {
                        return current_best(
                            terminated,
                            Some(BestAction {
                                path: Vec::new(),
                                action,
                                fitness,
                            }),
                        )
                    }
                    RateAction::NewBest(new) => best_action = Some(new),
                    RateAction::Worse(action) => actions.push(action),
                    RateAction::UpperLimit(action) => return Some(action),
                    RateAction::Terminated(action, fitness) => {
                        terminated.add_complete(action, fitness)
                    }
                    RateAction::PartiallyTerminated(action, fitness) => {
                        terminated.add_partial(action, fitness)
                    }
                }
            }

            for action in prev_actions.into_iter().rev() {
                let alpha = alpha(&terminated, &best_action);
                match self.rate_action(state, action, Vec::new(), alpha, depth, &mut condition) {
                    RateAction::Cancelled(action) => {
                        return current_best(terminated, best_action).or(Some(action))
                    }
                    RateAction::NewBest(new) => {
                        if let Some(prev_best) = best_action.replace(new) {
                            actions.push(prev_best.action)
                        }
                    }
                    RateAction::Worse(action) => actions.push(action),
                    RateAction::UpperLimit(action) => return Some(action),
                    RateAction::Terminated(action, fitness) => {
                        terminated.add_complete(action, fitness)
                    }
                    RateAction::PartiallyTerminated(action, fitness) => {
                        terminated.add_partial(action, fitness)
                    }
                }
            }

            // We only test partially terminated action which may still be better than the best
            // fitness at the current depth.
            //
            // As the current best fitness does not come from a terminated path,
            // we still have to keep the other partially terminated actions around,
            // in case the best fitness of a later depth is lower.
            for action in
                terminated.relevant_partials(best_action.as_ref().map(|best| best.fitness))
            {
                let alpha = alpha(&terminated, &best_action);
                match self.rate_action(state, action, Vec::new(), alpha, depth, &mut condition) {
                    RateAction::Cancelled(_action) => return current_best(terminated, best_action),
                    RateAction::NewBest(new) => {
                        if let Some(prev_best) = best_action.replace(new) {
                            actions.push(prev_best.action)
                        }
                    }
                    RateAction::Worse(action) => actions.push(action),
                    RateAction::UpperLimit(action) => return Some(action),
                    RateAction::Terminated(action, fitness) => {
                        terminated.add_complete(action, fitness)
                    }
                    RateAction::PartiallyTerminated(action, fitness) => {
                        terminated.add_partial(action, fitness)
                    }
                }
            }

            // All partially terminated actions are worse than all completely terminated actions.
            if actions.is_empty() && best_action.is_none() {
                assert!(terminated.partial.is_empty());
                return Some(terminated.best_action.map(|(a, _f)| a).unwrap());
            }
        }

        unreachable!();
    }

    fn rate_action<U: RunCondition>(
        &self,
        state: &T,
        action: T::Action,
        path: Vec<T::Action>,
        alpha: Option<T::Fitness>,
        depth: u32,
        condition: &mut U,
    ) -> RateAction<T> {
        let mut updated_state = state.clone();
        let fitness = updated_state.execute(&action, self.player);
        match self.minimax_with_path(path, updated_state, depth, alpha, None, condition) {
            Err(CancelledError) => RateAction::Cancelled(action),
            Ok(MiniMax::DeadEnd) => {
                if state.is_upper_bound(fitness, self.player) {
                    RateAction::UpperLimit(action)
                } else {
                    RateAction::Terminated(action, fitness)
                }
            }
            Ok(MiniMax::Terminated(_path, Branch::Equal(fitness))) => {
                if state.is_upper_bound(fitness, self.player) {
                    RateAction::UpperLimit(action)
                } else {
                    RateAction::Terminated(action, fitness)
                }
            }
            Ok(MiniMax::Terminated(_path, Branch::Worse(maximum))) => {
                RateAction::PartiallyTerminated(action, maximum)
            }
            Ok(MiniMax::Open(_path, Branch::Worse(_))) => RateAction::Worse(action),
            Ok(MiniMax::Open(path, Branch::Equal(fitness))) => {
                if Some(fitness) > alpha {
                    RateAction::NewBest(BestAction {
                        action,
                        fitness,
                        path,
                    })
                } else {
                    RateAction::Worse(action)
                }
            }
            Ok(MiniMax::Terminated(_, Branch::Better(_)))
            | Ok(MiniMax::Open(_, Branch::Better(_))) => {
                unreachable!("beta cutoff at highest depth");
            }
        }
    }

    /// Computes the next possible steps and sorts them to maximize
    /// cutoffs.
    fn generate_game_states(&self, game_state: &T) -> (bool, Vec<(T, T::Action, T::Fitness)>) {
        let (active, actions) = game_state.actions(self.player);

        let mut game_states: Vec<_> = actions
            .into_iter()
            .map(|action| {
                let mut game_state = game_state.clone();
                let fitness = game_state.execute(&action, self.player);
                (game_state, action, fitness)
            })
            .collect();

        if active {
            game_states.sort_unstable_by(|(_, _, a), (_, _, b)| a.cmp(b));
        } else {
            game_states.sort_unstable_by(|(_, _, a), (_, _, b)| b.cmp(a));
        }

        (active, game_states)
    }

    /// As we want to ignore as many possible subtrees as possible,
    /// we start each depth by taking the best possible path of the
    /// previous depth.
    ///
    /// As this path is hopefully also a good choice at this depth,
    /// we very quickly get a good alpha/lower limit.
    fn minimax_with_path<U: RunCondition>(
        &self,
        mut path: Vec<T::Action>,
        game_state: T,
        depth: u32,
        alpha: Option<T::Fitness>,
        beta: Option<T::Fitness>,
        condition: &mut U,
    ) -> Result<MiniMax<T>, CancelledError> {
        if !condition.step() {
            return Err(CancelledError);
        }

        match path.pop() {
            None => self.minimax(game_state, depth, alpha, beta, condition),
            Some(action) => {
                if depth == 0 {
                    unreachable!("lowest depth with non empty path");
                }

                let (active, mut game_states) = self.generate_game_states(&game_state);

                let mut state = State::new(game_state, self.player, alpha, None, active);
                match game_states.iter().position(|(_, a, _)| *a == action) {
                    Some(idx) => {
                        let (game_state, action, fitness) = game_states.remove(idx);

                        if let Some(cutoff) = state.bind(
                            self.minimax_with_path(
                                path,
                                game_state,
                                depth - 1,
                                state.alpha,
                                state.beta,
                                condition,
                            )?
                            .with(action, fitness),
                        ) {
                            return Ok(cutoff);
                        }
                    }
                    None => unreachable!("path segment not found"),
                }

                for (game_state, action, fitness) in game_states.into_iter().rev() {
                    if let Some(cutoff) = state.bind(
                        self.minimax(game_state, depth - 1, state.alpha, state.beta, condition)?
                            .with(action, fitness),
                    ) {
                        return Ok(cutoff);
                    }
                }

                Ok(state.consume())
            }
        }
    }

    fn minimax<U: RunCondition>(
        &self,
        game_state: T,
        depth: u32,
        alpha: Option<T::Fitness>,
        beta: Option<T::Fitness>,
        condition: &mut U,
    ) -> Result<MiniMax<T>, CancelledError> {
        if !condition.step() {
            Err(CancelledError)
        } else if depth == 0 {
            let (active, actions) = game_state.actions(self.player);
            let actions = actions.into_iter().map(|action| {
                let fitness = game_state.look_ahead(&action, self.player);
                (action, fitness)
            });
            let selected = if active {
                actions.max_by_key(|(_, fitness)| *fitness)
            } else {
                actions.min_by_key(|(_, fitness)| *fitness)
            };

            Ok(selected.map_or(MiniMax::DeadEnd, |(action, fitness)| {
                MiniMax::Open(vec![action], Branch::Equal(fitness))
            }))
        } else {
            let (active, game_states) = self.generate_game_states(&game_state);

            if game_states.is_empty() {
                return Ok(MiniMax::DeadEnd);
            }

            let mut state = State::new(game_state, self.player, alpha, beta, active);
            for (game_state, action, fitness) in game_states.into_iter().rev() {
                if let Some(cutoff) = state.bind(
                    self.minimax(game_state, depth - 1, state.alpha, state.beta, condition)?
                        .with(action, fitness),
                ) {
                    return Ok(cutoff);
                }
            }

            Ok(state.consume())
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct CancelledError;

enum MiniMax<T: Game> {
    /// No new elements were found in this branch
    Terminated(Vec<T::Action>, Branch<T>),
    /// New elements were found
    Open(Vec<T::Action>, Branch<T>),
    /// There are no possible actions for this state
    DeadEnd,
}

impl<T: Game> MiniMax<T> {
    /// Appends an action to self.
    pub fn with(self, action: T::Action, fitness: T::Fitness) -> MiniMax<T> {
        match self {
            MiniMax::DeadEnd => MiniMax::Terminated(vec![action], Branch::Equal(fitness)),
            MiniMax::Open(mut actions, branch) => {
                actions.push(action);
                MiniMax::Open(actions, branch)
            }
            MiniMax::Terminated(mut actions, branch) => {
                actions.push(action);
                MiniMax::Terminated(actions, branch)
            }
        }
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

enum Branch<T: Game> {
    /// `actual_fitness <= fitness`
    Worse(T::Fitness),
    /// `actual_fitness >= fitness`
    Better(T::Fitness),
    /// `actual_fitness == fitness`
    Equal(T::Fitness),
}

impl<T: Game> Branch<T> {
    fn fitness(&self) -> T::Fitness {
        match self {
            Branch::Worse(fitness) | Branch::Better(fitness) | Branch::Equal(fitness) => *fitness,
        }
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

struct State<T: Game> {
    state: T,
    player: T::Player,
    alpha: Option<T::Fitness>,
    beta: Option<T::Fitness>,
    best_fitness: Option<Branch<T>>,
    path: Vec<T::Action>,
    terminated: bool,
    active: bool,
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

impl<T: Game> State<T> {
    fn new(
        state: T,
        player: T::Player,
        alpha: Option<T::Fitness>,
        beta: Option<T::Fitness>,
        active: bool,
    ) -> Self {
        Self {
            state,
            player,
            alpha,
            beta,
            best_fitness: None,
            path: Vec::new(),
            terminated: true,
            active,
        }
    }

    fn update_best_action(&mut self, path: Vec<T::Action>, fitness: Branch<T>) {
        assert!(!path.is_empty());

        self.path = path;
        self.best_fitness = Some(fitness);
    }

    fn bind(&mut self, value: MiniMax<T>) -> Option<MiniMax<T>> {
        match value {
            MiniMax::DeadEnd => unreachable!(),
            MiniMax::Terminated(path, Branch::Equal(fitness)) => {
                self.bind_equal(path, fitness, true);
            }
            MiniMax::Terminated(path, Branch::Better(fitness)) => {
                self.bind_better(path, fitness, true);
            }
            MiniMax::Terminated(path, Branch::Worse(fitness)) => {
                self.bind_worse(path, fitness, true);
            }
            MiniMax::Open(path, Branch::Equal(fitness)) => {
                self.bind_equal(path, fitness, false);
            }
            MiniMax::Open(path, Branch::Better(fitness)) => {
                self.bind_better(path, fitness, false);
            }
            MiniMax::Open(path, Branch::Worse(fitness)) => {
                self.bind_worse(path, fitness, false);
            }
        }

        let branch = match (self.alpha, self.beta) {
            (Some(alpha), _) if self.active && self.state.is_upper_bound(alpha, self.player) => {
                Branch::Equal(alpha)
            }
            (_, Some(beta)) if !self.active && self.state.is_lower_bound(beta, self.player) => {
                Branch::Equal(beta)
            }
            (Some(alpha), Some(beta)) if alpha >= beta => {
                if self.active {
                    Branch::Better(self.alpha.unwrap())
                } else {
                    Branch::Worse(self.beta.unwrap())
                }
            }
            _ => return None,
        };

        if self.terminated {
            Some(MiniMax::Terminated(
                mem::replace(&mut self.path, Vec::new()),
                branch,
            ))
        } else {
            Some(MiniMax::Open(
                mem::replace(&mut self.path, Vec::new()),
                branch,
            ))
        }
    }

    fn bind_equal(&mut self, path: Vec<T::Action>, fitness: T::Fitness, terminated: bool) {
        self.terminated &= terminated;
        if self.active {
            if terminated && self.state.is_upper_bound(fitness, self.player) {
                self.alpha = Some(fitness);
                self.beta = Some(fitness);
                self.best_fitness = Some(Branch::Equal(fitness));
                self.terminated = true;
            } else {
                self.alpha = Some(self.alpha.map_or(fitness, |value| cmp::max(value, fitness)));
                if self
                    .best_fitness
                    .as_ref()
                    .map_or(true, |old| old.fitness() <= fitness)
                {
                    self.update_best_action(path, Branch::Equal(fitness));
                }
            }
        } else {
            if terminated && self.state.is_lower_bound(fitness, self.player) {
                self.alpha = Some(fitness);
                self.beta = Some(fitness);
                self.best_fitness = Some(Branch::Equal(fitness));
                self.terminated = true;
            } else {
                self.beta = Some(self.beta.map_or(fitness, |value| cmp::min(value, fitness)));
                if self
                    .best_fitness
                    .as_ref()
                    .map_or(true, |old| old.fitness() >= fitness)
                {
                    self.update_best_action(path, Branch::Equal(fitness));
                }
            }
        }
    }

    fn bind_better(&mut self, path: Vec<T::Action>, fitness: T::Fitness, terminated: bool) {
        self.terminated &= terminated;
        if self.active {
            debug_assert!(self.alpha.map_or(true, |value| value <= fitness));
            self.alpha = Some(fitness);
            debug_assert!(self
                .best_fitness
                .as_ref()
                .map_or(true, |value| value.fitness() <= fitness));
            self.update_best_action(path, Branch::Better(fitness));
        } else if self
            .best_fitness
            .as_ref()
            .map_or(true, |old| old.fitness() > fitness)
        {
            self.update_best_action(path, Branch::Better(fitness))
        }
    }

    fn bind_worse(&mut self, path: Vec<T::Action>, fitness: T::Fitness, terminated: bool) {
        self.terminated &= terminated;
        if !self.active {
            debug_assert!(self.beta.map_or(true, |value| value >= fitness));
            self.beta = Some(fitness);
            debug_assert!(self
                .best_fitness
                .as_ref()
                .map_or(true, |value| value.fitness() >= fitness));
            self.update_best_action(path, Branch::Worse(fitness));
        } else if self
            .best_fitness
            .as_ref()
            .map_or(true, |old| old.fitness() < fitness)
        {
            self.update_best_action(path, Branch::Worse(fitness))
        }
    }

    fn consume(self) -> MiniMax<T> {
        if self.terminated {
            MiniMax::Terminated(self.path, self.best_fitness.unwrap())
        } else {
            MiniMax::Open(self.path, self.best_fitness.unwrap())
        }
    }
}

struct BestAction<T: Game> {
    action: T::Action,
    fitness: T::Fitness,
    path: Vec<T::Action>,
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

/// Contains data about already terminated paths.
struct Terminated<T: Game> {
    /// The fitness of the best completely finished action.
    best_action: Option<(T::Action, T::Fitness)>,
    /// Actions which terminated due to a cutoff, meaning that `fitness >= actual fitness`.
    partial: Vec<(T::Action, T::Fitness)>,
}

impl<T: Game> Default for Terminated<T> {
    #[inline]
    fn default() -> Self {
        Terminated {
            best_action: None,
            partial: Vec::new(),
        }
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

impl<T: Game> Terminated<T> {
    /// Returns all partially terminated actions which might be better than `best_fitness`.
    #[inline]
    fn relevant_partials(&mut self, best_fitness: Option<T::Fitness>) -> Vec<T::Action> {
        let mut relevant = Vec::new();
        for (action, fitness) in mem::replace(&mut self.partial, Vec::new()) {
            if Some(fitness) > best_fitness {
                relevant.push(action);
            } else {
                self.partial.push((action, fitness));
            }
        }
        relevant
    }

    fn add_complete(&mut self, action: T::Action, fitness: T::Fitness) {
        if self.best_fitness().map_or(true, |best| best < fitness) {
            self.best_action = Some((action, fitness));
            self.partial.retain(|(_a, f)| *f > fitness);
        }
    }

    fn add_partial(&mut self, action: T::Action, fitness: T::Fitness) {
        if self.best_fitness().map_or(true, |best| best < fitness) {
            self.partial.push((action, fitness));
        }
    }

    /// returns the fitness of the best completely terminated action
    fn best_fitness(&self) -> Option<T::Fitness> {
        self.best_action.as_ref().map(|(_a, fitness)| *fitness)
    }
}

fn current_best<T: Game>(
    terminated: Terminated<T>,
    best_action: Option<BestAction<T>>,
) -> Option<T::Action> {
    match (terminated.best_action, best_action) {
        (Some(term), Some(best)) => Some(if best.fitness > term.1 {
            best.action
        } else {
            term.0
        }),
        (Some(term), None) => Some(term.0),
        (None, Some(best)) => Some(best.action),
        (None, None) => None,
    }
}

#[inline]
fn alpha<T: Game>(
    terminated: &Terminated<T>,
    best_action: &Option<BestAction<T>>,
) -> Option<T::Fitness> {
    cmp::max(
        best_action.as_ref().map(|best| best.fitness),
        terminated.best_fitness(),
    )
}

enum RateAction<T: Game> {
    Cancelled(T::Action),
    NewBest(BestAction<T>),
    Worse(T::Action),
    UpperLimit(T::Action),
    Terminated(T::Action, T::Fitness),
    PartiallyTerminated(T::Action, T::Fitness),
}

impl<T: Game> Debug for RateAction<T>
where
    T::Action: Debug,
    T::Fitness: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RateAction::Cancelled(action) => write!(f, "Cancelled({:?})", action),
            RateAction::NewBest(best_action) => write!(f, "NewBest({:?})", best_action),
            RateAction::Worse(action) => write!(f, "Worse({:?})", action),
            RateAction::UpperLimit(action) => write!(f, "UpperLimit({:?})", action),
            RateAction::Terminated(action, fitness) => {
                write!(f, "Terminated({:?}, {:?})", action, fitness)
            }
            RateAction::PartiallyTerminated(action, fitness) => {
                write!(f, "PartiallyTerminated({:?}, {:?})", action, fitness)
            }
        }
    }
}
