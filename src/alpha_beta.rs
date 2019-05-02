//! A deterministic game bot using alpha beta pruning.

use crate::{Game, IntoRunCondition, RunCondition};

use std::cmp;
use std::fmt::{self, Debug};
use std::mem;

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
    fn new(alpha: Option<T::Fitness>, beta: Option<T::Fitness>, active: bool) -> Self {
        Self {
            alpha,
            beta,
            best_fitness: None,
            path: Vec::new(),
            terminated: true,
            active,
        }
    }

    fn update_best_action(&mut self, path: Vec<T::Action>, action: T::Action, fitness: Branch<T>) {
        self.path = path;
        self.path.push(action);
        self.best_fitness = Some(fitness);
    }

    fn bind_equal(
        &mut self,
        path: Vec<T::Action>,
        fitness: T::Fitness,
        action: T::Action,
        terminated: bool,
    ) {
        self.terminated &= terminated;
        if self.active {
            if terminated && T::UPPER_LIMIT.map_or(false, |limit| fitness >= limit) {
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
                    self.update_best_action(path, action, Branch::Equal(fitness));
                }
            }
        } else {
            if terminated && T::LOWER_LIMIT.map_or(false, |limit| fitness <= limit) {
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
                    self.update_best_action(path, action, Branch::Equal(fitness));
                }
            }
        }
    }

    fn bind_better(
        &mut self,
        path: Vec<T::Action>,
        fitness: T::Fitness,
        action: T::Action,
        terminated: bool,
    ) {
        self.terminated &= terminated;
        if self.active {
            debug_assert!(self.alpha.map_or(true, |value| value <= fitness));
            self.alpha = Some(fitness);
            debug_assert!(self
                .best_fitness
                .as_ref()
                .map_or(true, |value| value.fitness() <= fitness));
            self.update_best_action(path, action, Branch::Better(fitness));
        } else if self
            .best_fitness
            .as_ref()
            .map_or(true, |old| old.fitness() >= fitness)
        {
            self.update_best_action(path, action, Branch::Better(fitness))
        }
    }

    fn bind_worse(
        &mut self,
        path: Vec<T::Action>,
        fitness: T::Fitness,
        action: T::Action,
        terminated: bool,
    ) {
        self.terminated &= terminated;
        if !self.active {
            debug_assert!(self.beta.map_or(true, |value| value >= fitness));
            self.beta = Some(fitness);
            debug_assert!(self
                .best_fitness
                .as_ref()
                .map_or(true, |value| value.fitness() >= fitness));
            self.update_best_action(path, action, Branch::Worse(fitness));
        } else if self
            .best_fitness
            .as_ref()
            .map_or(true, |old| old.fitness() <= fitness)
        {
            self.update_best_action(path, action, Branch::Worse(fitness))
        }
    }

    fn cutoff(&mut self) -> Option<MiniMax<T>> {
        match (self.alpha, self.beta) {
            (Some(alpha), Some(beta)) if alpha >= beta => {
                let branch = if self.active {
                    if T::UPPER_LIMIT.map_or(false, |limit| alpha >= limit) {
                        Branch::Equal(alpha)
                    } else {
                        Branch::Better(self.alpha.unwrap())
                    }
                } else {
                    if T::LOWER_LIMIT.map_or(false, |limit| beta <= limit) {
                        Branch::Equal(beta)
                    } else {
                        Branch::Worse(self.beta.unwrap())
                    }
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
            _ => None,
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

/// contains data about already terminated paths
struct Terminated<T: Game> {
    /// the fitness of the best completely finished action
    best_action: Option<(T::Action, T::Fitness)>,
    /// actions which terminated due to a cutoff, meaning that `fitness >= actual fitness`
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
    /// returns all partially terminated actions which might be better than `best_fitness`
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
    Terminated,
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
            RateAction::Terminated => write!(f, "Terminated"),
        }
    }
}

/// A game bot which analyses its moves using alpha beta pruning with iterative deepening. In case [`select`][sel] terminates
/// before `condition` returned true, the result is always the best possible move. While this bot caches some data
/// during computation, it does not require a lot of memory and will not store anything between different [`select`][sel] calls.
///
/// This bot requires [`Game`][game] to be implemented for your game.
///
/// [sel]: trait.GameBot.html#tymethod.select
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
    /// In case no `Action` is possible or the bot is currently not the active player, this functions returns `None`.
    /// This method runs until either the best possible action was found
    /// or one of `RunCondition::depth` and `RunCondition::step` returned `false`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rubot::{Bot, ToCompletion, tree::Node};
    /// use std::time::Duration;
    ///
    /// const TREE: Node = Node::root().children(&[
    ///     Node::new(false, 7).children(&[
    ///         Node::new(true, 4),
    ///         Node::new(true, 2),
    ///     ]),
    ///     Node::new(false, 5).children(&[
    ///         Node::new(true, 8),
    ///         Node::new(true, 9)
    ///     ]),
    ///     Node::new(false, 6),
    /// ]);
    ///
    /// let mut bot = Bot::new(true);
    ///
    /// // finds the best possible action
    /// let best = bot.select(&TREE, ToCompletion);
    /// // searches for at most 2 seconds and returns the best answer found.
    /// // As 2 seconds are more than enough for this simple tree, this will
    /// // return the best possible action without spending this much time
    /// let limited = bot.select(&TREE, Duration::from_secs(2));
    ///
    /// assert_eq!(best, Some(1));
    /// assert_eq!(limited, Some(1));
    /// ```
    pub fn select<U: IntoRunCondition>(&mut self, state: &T, condition: U) -> Option<T::Action> {
        let mut condition = condition.into_run_condition();

        let (active, actions) = state.actions(&self.player);
        if !active {
            return None;
        }

        let mut actions = actions.into_iter().collect::<Vec<_>>();
        if actions.len() < 2 {
            return actions.pop();
        }
        actions.sort_by_cached_key(|a| state.look_ahead(&a, &self.player));

        // the best action which already terminated
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

                match self.rate_action(
                    state,
                    action,
                    &mut terminated,
                    path,
                    alpha,
                    depth,
                    &mut condition,
                ) {
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
                    RateAction::Terminated => (),
                }
            }

            for action in prev_actions.into_iter().rev() {
                let alpha = alpha(&terminated, &best_action);
                match self.rate_action(
                    state,
                    action,
                    &mut terminated,
                    Vec::new(),
                    alpha,
                    depth,
                    &mut condition,
                ) {
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
                    RateAction::Terminated => (),
                }
            }

            for action in
                terminated.relevant_partials(best_action.as_ref().map(|best| best.fitness))
            {
                let alpha = alpha(&terminated, &best_action);
                match self.rate_action(
                    state,
                    action,
                    &mut terminated,
                    Vec::new(),
                    alpha,
                    depth,
                    &mut condition,
                ) {
                    RateAction::Cancelled(_action) => return current_best(terminated, best_action),
                    RateAction::NewBest(new) => {
                        if let Some(prev_best) = best_action.replace(new) {
                            actions.push(prev_best.action)
                        }
                    }
                    RateAction::Worse(action) => actions.push(action),
                    RateAction::UpperLimit(action) => return Some(action),
                    RateAction::Terminated => (),
                }
            }

            // all partially terminated actions are worse than all completely terminated actions
            if actions.is_empty() && best_action.is_none() {
                assert!(terminated.partial.is_empty());
                break;
            }
        }

        // all branches are terminated, as the loop is finished
        Some(terminated.best_action.map(|(a, _f)| a).unwrap())
    }

    fn rate_action<U: RunCondition>(
        &mut self,
        state: &T,
        action: T::Action,
        terminated: &mut Terminated<T>,
        path: Vec<T::Action>,
        alpha: Option<T::Fitness>,
        depth: u32,
        condition: &mut U,
    ) -> RateAction<T> {
        let mut state = state.clone();
        let fitness = state.execute(&action, &self.player);
        match self.minimax(path, state, depth, alpha, None, condition) {
            Err(CancelledError) => RateAction::Cancelled(action),
            Ok(MiniMax::DeadEnd) => {
                terminated.add_complete(action, fitness);
                RateAction::Terminated
            }
            Ok(MiniMax::Terminated(_path, Branch::Equal(fitness))) => {
                if T::UPPER_LIMIT.map_or(false, |limit| fitness >= limit) {
                    RateAction::UpperLimit(action)
                } else {
                    terminated.add_complete(action, fitness);
                    RateAction::Terminated
                }
            }
            Ok(MiniMax::Terminated(_path, Branch::Worse(maximum))) => {
                terminated.add_partial(action, maximum);
                RateAction::Terminated
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
            Ok(MiniMax::Terminated(_path, Branch::Better(_))) => {
                unreachable!("terminated beta cutoff at highest depth");
            }
            Ok(MiniMax::Open(_path, Branch::Better(_))) => {
                unreachable!("open beta cutoff at highest depth");
            }
        }
    }

    fn minimax<U: RunCondition>(
        &mut self,
        mut path: Vec<T::Action>,
        game_state: T,
        depth: u32,
        alpha: Option<T::Fitness>,
        beta: Option<T::Fitness>,
        condition: &mut U,
    ) -> Result<MiniMax<T>, CancelledError> {
        if !condition.step() {
            Err(CancelledError)
        } else if depth == 0 {
            debug_assert!(
                path.is_empty(),
                "The previous search should not have reached this deep"
            );

            let (active, actions) = game_state.actions(&self.player);
            let actions = actions.into_iter().map(|action| {
                let fitness = game_state.look_ahead(&action, &self.player);
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
            let (active, actions) = game_state.actions(&self.player);
            let mut states: Vec<_> = actions
                .into_iter()
                .filter(|action| path.last().map_or(true, |path| path != action))
                .map(|action| {
                    let mut game_state = game_state.clone();
                    let fitness = game_state.execute(&action, &self.player);
                    (game_state, action, fitness)
                })
                .collect();

            if active {
                states.sort_unstable_by(|(_, _, a), (_, _, b)| a.cmp(b));
            } else {
                states.sort_unstable_by(|(_, _, a), (_, _, b)| b.cmp(a));
            }

            if let Some(action) = path.pop() {
                let mut game_state = game_state.clone();
                let fitness = game_state.execute(&action, &self.player);
                states.push((game_state, action, fitness))
            }

            if states.is_empty() {
                return Ok(MiniMax::DeadEnd);
            }

            let mut state = State::new(alpha, beta, active);
            for (game_state, action, fitness) in states.into_iter().rev() {
                if let Some(cutoff) = state.cutoff() {
                    return Ok(cutoff);
                }

                match self.minimax(
                    mem::replace(&mut path, Vec::new()),
                    game_state,
                    depth - 1,
                    alpha,
                    beta,
                    condition,
                )? {
                    MiniMax::DeadEnd => {
                        state.bind_equal(Vec::new(), fitness, action, true);
                    }
                    MiniMax::Terminated(path, Branch::Equal(fitness)) => {
                        state.bind_equal(path, fitness, action, true);
                    }
                    MiniMax::Terminated(path, Branch::Better(fitness)) => {
                        state.bind_better(path, fitness, action, true);
                    }
                    MiniMax::Terminated(path, Branch::Worse(fitness)) => {
                        state.bind_worse(path, fitness, action, true);
                    }
                    MiniMax::Open(path, Branch::Equal(fitness)) => {
                        state.bind_equal(path, fitness, action, false);
                    }
                    MiniMax::Open(path, Branch::Better(fitness)) => {
                        state.bind_better(path, fitness, action, false);
                    }
                    MiniMax::Open(path, Branch::Worse(fitness)) => {
                        state.bind_worse(path, fitness, action, false);
                    }
                }
            }

            Ok(state.consume())
        }
    }
}
