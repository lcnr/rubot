//! A deterministic game bot using alpha beta pruning.

use crate::{Game, IntoRunCondition, RunCondition};

use std::cmp;
use std::fmt::{self, Debug};
use std::mem;

struct CancelledError;

enum MiniMax<T: Game> {
    /// No new elements were found in this branch
    Terminated(Vec<T::Action>, Branch<T>),
    /// New elements were found
    Open(Vec<T::Action>, Branch<T>),
    /// There are no possible actions for this state
    DeadEnd,
}

enum Branch<T: Game> {
    /// `actual_fitness <= fitness`
    Worse(T::Fitness),
    /// `actual_fitness >= fitness`
    Better(T::Fitness),
    /// `actual_fitness == fitness`
    Equal(T::Fitness),
}

struct State<T: Game> {
    alpha: Option<T::Fitness>,
    beta: Option<T::Fitness>,
    best_fitness: Option<T::Fitness>,
    path: Vec<T::Action>,
    terminated: bool,
    active: bool,
    /// this is either less than alpha or more than beta, used if all actions are *undesirable*
    edge_case: Option<T::Fitness>,
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
            edge_case: None,
        }
    }

    fn update_best_action(&mut self, path: Vec<T::Action>, action: T::Action, fitness: T::Fitness) {
        self.path = path;
        self.path.push(action);
        self.best_fitness = Some(fitness)
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
            self.alpha = Some(self.alpha.map_or(fitness, |value| cmp::max(value, fitness)));
            if self.best_fitness.map_or(true, |old| old <= fitness) {
                self.update_best_action(path, action, fitness);
            }
        } else {
            self.beta = Some(self.beta.map_or(fitness, |value| cmp::min(value, fitness)));
            if self.best_fitness.map_or(true, |old| old >= fitness) {
                self.update_best_action(path, action, fitness);
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
            debug_assert!(self.best_fitness.map_or(true, |value| value <= fitness));
            self.update_best_action(path, action, fitness);
        } else if self.best_fitness.is_none() {
            self.edge_case = Some(self.edge_case.map_or(fitness, |min| cmp::min(fitness, min)));
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
            debug_assert!(self.best_fitness.map_or(true, |value| value >= fitness));
            self.update_best_action(path, action, fitness);
        } else if self.best_fitness.is_none() {
            self.edge_case = Some(self.edge_case.map_or(fitness, |max| cmp::max(fitness, max)));
        }
    }

    fn is_cutoff(&self) -> bool {
        if let (Some(ref alpha), Some(ref beta)) = (self.alpha, self.beta) {
            alpha >= beta
        } else {
            false
        }
    }

    fn consume(self) -> MiniMax<T> {
        let branch = match (self.is_cutoff(), self.active) {
            (true, true) => Branch::Better(self.alpha.unwrap()),
            (true, false) => Branch::Worse(self.beta.unwrap()),
            (false, _) => self
                .best_fitness
                .map(|res| Branch::Equal(res))
                .unwrap_or_else(|| {
                    if self.active {
                        Branch::Worse(self.edge_case.unwrap())
                    } else {
                        Branch::Better(self.edge_case.unwrap())
                    }
                }),
        };

        if self.terminated {
            MiniMax::Terminated(self.path, branch)
        } else {
            MiniMax::Open(self.path, branch)
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
            self.partial.retain(|(_a, f)| f > &fitness);
        }
    }

    fn add_partial(&mut self, action: T::Action, fitness: T::Fitness) {
        if self.best_fitness().map_or(true, |best| best < fitness) {
            self.partial.push((action, fitness));
        }
    }

    fn best_fitness(&self) -> Option<T::Fitness> {
        self.best_action.as_ref().map(|(_a, fitness)| *fitness)
    }

    fn finalize(self) -> T::Action {
        self.best_action.map(|(a, _f)| a).unwrap()
    }
}

fn current_best<T: Game>(terminated: Terminated<T>, best_action: Option<BestAction<T>>) -> Option<T::Action> {
    match (terminated.best_action, best_action) {
        (Some(term), Some(best)) => Some(if best.fitness > term.1 { best.action } else { term.0 }),
        (Some(term), None) => Some(term.0),
        (None, Some(best)) => Some(best.action),
        (None, None) => None,
    }
}

enum RateAction<T: Game> {
    Cancelled(T::Action),
    NewBest(BestAction<T>),
    Worse(T::Action),
    Terminated,
}

/// A game bot which analyses its moves using alpha beta pruning with iterative deepening. In case [`select`][sel] terminates
/// after less than `duration`, the result is always the best possible move. While this bot does cache some data
/// during computation, it does not require a lot of memory and does not store anything between different [`select`][sel] calls.
///
/// [sel]:trait.GameBot.html#tymethod.select
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
    /// or one of `fn RunCondition::depth` and `fn RunCondition::step` returned `false`.
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
                return current_best(terminated, best_action)
            }

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
                        return current_best(terminated, Some(BestAction { path: Vec::new(), action, fitness }))
                    }
                    RateAction::NewBest(new) => best_action = Some(new),
                    RateAction::Worse(action) => actions.push(action),
                    RateAction::Terminated => (),
                }
            }

            for action in mem::replace(&mut actions, Vec::new()).into_iter().rev() {
                let alpha = cmp::max(
                    best_action.as_ref().map(|best| best.fitness),
                    terminated.best_fitness(),
                );
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
                        best_action
                            .replace(new)
                            .map(|prev| actions.push(prev.action));
                    }
                    RateAction::Worse(action) => actions.push(action),
                    RateAction::Terminated => (),
                }
            }

            for action in
                terminated.relevant_partials(best_action.as_ref().map(|best| best.fitness))
            {
                let alpha = cmp::max(
                    best_action.as_ref().map(|best| best.fitness),
                    terminated.best_fitness(),
                );
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
                        best_action
                            .replace(new)
                            .map(|prev| actions.push(prev.action));
                    }
                    RateAction::Worse(action) => actions.push(action),
                    RateAction::Terminated => (),
                }
            }

            // all partially terminated actions are worse than all completely terminated actions
            if actions.is_empty() && best_action.is_none() {
                debug_assert!(terminated.partial.is_empty());
                break;
            }
        }

        // all branches are terminated, as the loop is finished
        Some(terminated.finalize())
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
                terminated.add_complete(action, fitness);
                RateAction::Terminated
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
            Ok(MiniMax::Terminated(_path, Branch::Better(_)))
            | Ok(MiniMax::Open(_path, Branch::Better(_))) => {
                unreachable!("beta cutoff at highest depth");
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
            let selected = if active {
                actions
                    .into_iter()
                    .map(|action| {
                        let fitness = game_state.look_ahead(&action, &self.player);
                        (action, fitness)
                    })
                    .max_by_key(|(_, fitness)| *fitness)
            } else {
                actions
                    .into_iter()
                    .map(|action| {
                        let fitness = game_state.look_ahead(&action, &self.player);
                        (action, fitness)
                    })
                    .min_by_key(|(_, fitness)| *fitness)
            };

            Ok(selected
                .map(|(action, fitness)| MiniMax::Open(vec![action], Branch::Equal(fitness)))
                .unwrap_or(MiniMax::DeadEnd))
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

            states.sort_unstable_by_key(|(_, _, fitness)| *fitness);
            path.pop().map(|action| {
                let mut game_state = game_state.clone();
                let fitness = game_state.execute(&action, &self.player);
                states.push((game_state, action, fitness))
            });

            if states.is_empty() {
                return Ok(MiniMax::DeadEnd);
            }

            let mut state = State::new(alpha, beta, active);
            for (game_state, action, fitness) in states.into_iter().rev() {
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

                if state.is_cutoff() {
                    break;
                }
            }

            Ok(state.consume())
        }
    }
}
