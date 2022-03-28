//! A deterministic game bot using alpha beta pruning.
use crate::{Game, IntoRunCondition, RunCondition};

use tapir::Tap;

use std::cmp::{self, Reverse};
use std::mem;

mod debug;

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
        self.inner_select(state, condition)
            .map(|mut act| act.path.pop().unwrap())
    }

    /// Similar to `select`, except that this function also returns the principal variation and the
    /// final evaluation of the given action.
    ///
    /// The actions are sorted in order they are executed, so
    /// `action.path[0]` is always equal to the result of `select`.
    ///
    /// ```rust
    /// use rubot::{Bot, ToCompletion, tree::Node};
    ///
    /// # #[rustfmt::skip]
    /// let tree = Node::root().with_children(&[
    ///     Node::new(true, 4),
    ///     Node::new(true, 0).with_children(&[
    ///         Node::new(true, 5), // This is the best possible result.
    ///         Node::new(true, 3),
    ///     ])
    /// ]);
    ///
    /// assert_eq!(&Bot::new(true)
    ///     .detailed_select(&tree, ToCompletion)
    ///     .unwrap()
    ///     .path, &[1, 0]);
    /// ```
    pub fn detailed_select<U: IntoRunCondition>(
        &mut self,
        state: &T,
        condition: U,
    ) -> Option<Action<T>> {
        self.inner_select(state, condition)
            .map(|act| act.tap(|act| act.path.reverse()))
    }

    fn inner_select<U: IntoRunCondition>(&mut self, state: &T, condition: U) -> Option<Action<T>> {
        let mut condition = condition.into_run_condition();

        let (active, actions) = state.actions(self.player);
        if !active {
            return None;
        }

        let actions: Vec<_> = actions
            .into_iter()
            .map(|action| Action {
                fitness: state.look_ahead(&action, self.player),
                path: vec![action],
            })
            .collect();

        if actions.is_empty() {
            return None;
        }

        let mut ctxt = Ctxt::new(state, self.player, actions);

        for depth in 0.. {
            if !condition.depth(depth) {
                return Some(ctxt.cancel());
            }

            // Return early in case there is only one relevant action left.
            // This is the case if we either only have one possible actions,
            // or if all other possible actions are worse than the lower bound.
            if let Some(exhausted) = ctxt.exhausted() {
                return Some(exhausted);
            }

            let mut unfinished = mem::take(&mut ctxt.unfinished);
            // Try unfinished actions with a high expected fitness first,
            // as they are expected to give us a better alpha value.
            unfinished.sort_by_key(|act| Reverse(act.fitness));

            if let Some(best) = ctxt.best.take() {
                // If computation is cancelled here, we don't know anything new,
                // so we can just return the previous best action.
                if let Some(ret) = ctxt.try_action(best, depth, &mut condition, |_, act| act) {
                    return Some(ret);
                }
            }

            for action in unfinished.into_iter() {
                // In case computation is cancelled here, we may not yet have computed the best action of
                // the previous depth, to guard against this, we add the cancelled action back to `state.unfinished`
                // in case it is still empty.
                let on_cancel = |ctxt: &mut Ctxt<T>, act| {
                    if ctxt.unfinished.is_empty() {
                        ctxt.unfinished.push(act);
                    }
                    ctxt.cancel()
                };

                if let Some(ret) = ctxt.try_action(action, depth, &mut condition, on_cancel) {
                    return Some(ret);
                }
            }

            // We only test partially terminated action which may still be better than the best
            // fitness at the current depth.
            //
            // As the current best fitness does not come from a terminated path,
            // we still have to keep the other partially terminated actions around,
            // in case the best fitness of a later depth is lower.
            for action in ctxt.relevant_partials() {
                // In case computation is cancelled here, we already tested at least some actions which were better than
                // the cancelled partial action at the previous depth, so we can use `ctxt.cancel()` without any special
                // considerations.
                if let Some(ret) =
                    ctxt.try_action(action, depth, &mut condition, |ctxt, _| ctxt.cancel())
                {
                    return Some(ret);
                }
            }
        }

        unreachable!();
    }
}

/// A top level action.
pub struct Action<T: Game> {
    /// The current fitness of a given action.
    ///
    /// This can mean one of the following things:
    ///
    /// - For the best unfinished action, this is exact, but only for the current depth.
    /// - For a terminated action, this is exact.
    /// - For a partially terminated action, this is the upper limit.
    pub fitness: T::Fitness,
    /// The expected path taken during optimal play, when only  inspecting up to the current depth.
    ///
    /// This used as a stack, with `path.pop()` being the first action.
    pub path: Vec<T::Action>,
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
    pub fn with(
        self,
        ctxt: &mut Ctxt<'_, T>,
        action: T::Action,
        fitness: T::Fitness,
    ) -> MiniMax<T> {
        match self {
            MiniMax::DeadEnd => MiniMax::Terminated(
                ctxt.new_path().tap(|p| p.push(action)),
                Branch::Equal(fitness),
            ),
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

/// A fitness and how it was calculated,
/// this is used if we want to know whether a cutoff occurred.
enum Branch<T: Game> {
    /// `actual_fitness <= fitness`.
    ///
    /// Used if the given branch is worse than the current `beta`.
    /// Confusingly, this is also called an `alpha` cutoff.
    Worse(T::Fitness),
    /// `actual_fitness >= fitness`.
    ///
    /// Used if the given branch is better than the current `alpha`.
    /// Confusingly, this is also called a `beta` cutoff.
    Better(T::Fitness),
    /// `actual_fitness == fitness`.
    ///
    /// Used if no cutoff occured.
    Equal(T::Fitness),
}

impl<T: Game> Clone for Branch<T> {
    fn clone(&self) -> Branch<T> {
        *self
    }
}

impl<T: Game> Copy for Branch<T> {}

impl<T: Game> Branch<T> {
    #[inline(always)]
    fn fitness(self) -> T::Fitness {
        match self {
            Branch::Worse(fitness) | Branch::Better(fitness) | Branch::Equal(fitness) => fitness,
        }
    }
}

/// The currently available data at the highest level, during minimax `State` is used instead.
struct Ctxt<'a, T: Game> {
    /// The initial gamestate.
    state: &'a T,
    /// The maximizing player.
    player: T::Player,
    /// The best unfinished action. This is not set if there is an already better terminated action.
    best: Option<Action<T>>,
    /// Actions which are both not yet finished and worse than `best_unfinished`.
    unfinished: Vec<Action<T>>,
    /// The best already completely terminated action. We keep the path for diagnostic
    /// purposes only, as there is no reason to retry this.
    terminated: Option<Action<T>>,
    /// Partially terminated actions. These are paths which had a cutoff at the highest level.
    ///
    /// As these actions cannot have a fitness higher than this cutoff, we discard all partially terminated
    /// actions which must be worse than `best_terminated`.
    partially_terminated: Vec<Action<T>>,
    /// In case all paths lead to defeat, we store the action which takes the longest,
    /// so the bot doesn't start doing weird stuff once it realized it's lost.
    losing_action: Option<Action<T>>,
    /// We create and discard a lot of paths.
    ///
    /// As an optimization, we therefore can reuse these paths.
    /// The paths stored here are always empty. This causes an about
    /// 2% performance increase.
    path_cache: Vec<Vec<T::Action>>,
}

impl<'a, T: Game> Ctxt<'a, T> {
    fn new(state: &T, player: T::Player, unfinished: Vec<Action<T>>) -> Ctxt<T> {
        Ctxt {
            state,
            player,
            best: None,
            unfinished,
            terminated: None,
            losing_action: None,
            partially_terminated: Vec::new(),
            path_cache: Vec::new(),
        }
    }

    /// Creates a new empty path, potentially reuse the cache.
    #[inline(always)]
    pub fn new_path(&mut self) -> Vec<T::Action> {
        // While it would be possible to create new paths using `Vec::with_capacity(depth)`
        // here, this does not actually influence the benchmarks so I decided against it.
        self.path_cache.pop().unwrap_or_else(Vec::new)
    }

    /// Discards a path, storing it in the cache.
    #[inline(always)]
    pub fn discard_path(&mut self, mut path: Vec<T::Action>) {
        // Note that `path.clear()` does not free the allocated storage.
        path.clear();
        self.path_cache.push(path);
    }

    /// Returns all partially terminated actions may be better than `self.best_unfinished`,
    /// and should therefore be retried at the current depth.
    fn relevant_partials(&mut self) -> impl IntoIterator<Item = Action<T>> {
        self.partially_terminated.sort_by_key(|act| act.fitness);

        if let Some(ref best) = self.best {
            // We only care about partially terminated paths which may be better than the current best.
            let pos = self
                .partially_terminated
                .iter()
                .position(|act| act.fitness > best.fitness)
                .unwrap_or(self.partially_terminated.len());
            self.partially_terminated.split_off(pos)
        } else {
            mem::take(&mut self.partially_terminated)
        }
    }

    /// Updates `self.terminated` in case the new action has a higher fitness.
    ///
    /// This also removes all partially terminated actions with a worse maximum fitness,
    /// as they are now irrelevant.
    fn add_terminated(&mut self, act: Action<T>) {
        if self
            .terminated
            .as_ref()
            .map_or(true, |best| best.fitness < act.fitness)
        {
            // Remove a partially terminated which are worse than the new best terminated action.
            //
            // This pretty much a manual reimplementation of `Vec::drain_filter`, which is currently unstable.
            for i in (0..self.partially_terminated.len()).rev() {
                if self.partially_terminated[i].fitness <= act.fitness {
                    let act = self.partially_terminated.swap_remove(i);
                    self.discard_path(act.path);
                }
            }

            // `best` is expected to always be better than `terminated`.
            if let Some(best) = self.best.take() {
                if best.fitness > act.fitness {
                    // Still relevant, put it back in.
                    self.best = Some(best);
                } else {
                    // Not relevant, add it to the other unfinished actions.
                    self.unfinished.push(best);
                }
            }

            if let Some(term) = self.terminated.replace(act) {
                self.discard_path(term.path);
            }
        } else {
            self.discard_path(act.path);
        }
    }

    /// Adds a new partially finished action in case its maximum fitness is
    /// greater than the fitness of the best completely terminated action.
    fn add_partially_terminated(&mut self, act: Action<T>) {
        if self
            .terminated
            .as_ref()
            .map_or(true, |best| best.fitness < act.fitness)
        {
            self.partially_terminated.push(act);
        } else {
            self.discard_path(act.path);
        }
    }

    fn add_best(&mut self, act: Action<T>) {
        if self
            .best
            .as_ref()
            .or(self.terminated.as_ref())
            .map_or(true, |best| best.fitness < act.fitness)
        {
            // Move the previous best action back into `unfinished`.
            self.unfinished.extend(self.best.replace(act));
        } else {
            self.unfinished.push(act);
        }
    }

    /// Stop computing and return the currently best action.
    fn cancel(&mut self) -> Action<T> {
        self.best
            .take()
            .or(self.terminated.take())
            .or_else(|| {
                mem::take(&mut self.unfinished)
                    .into_iter()
                    .max_by_key(|act| act.fitness)
            })
            .unwrap_or_else(|| {
                // In case no other action exists,
                // we need at least one guaranteed losing action.
                self.losing_action.take().unwrap()
            })
    }

    fn exhausted(&mut self) -> Option<Action<T>> {
        if self.best.is_none() && self.unfinished.is_empty() {
            // We can only get partially terminated actions in
            // case there is a better non terminated one.
            assert!(self.partially_terminated.is_empty());

            Some(
                self.terminated
                    .take()
                    .unwrap_or_else(|| self.losing_action.take().unwrap()),
            )
        } else {
            None
        }
    }

    /// Tests the given action at the current depth, returns `Some`
    /// once we are finished.
    fn try_action<U: RunCondition>(
        &mut self,
        mut action: Action<T>,
        depth: u32,
        condition: &mut U,
        on_cancel: impl FnOnce(&mut Self, Action<T>) -> Action<T>,
    ) -> Option<Action<T>> {
        let mut updated_state = self.state.clone();
        let (start, rest) = action.path.split_last().expect("unexpected empty path");

        let fitness = updated_state.execute(start, self.player);
        match self.minimax_with_path(
            rest.iter().cloned().rev(),
            updated_state,
            depth,
            self.best
                .as_ref()
                .or(self.terminated.as_ref())
                .map(|act| act.fitness),
            None,
            condition,
        ) {
            Err(CancelledError) => Some(on_cancel(self, action)),
            Ok(MiniMax::DeadEnd) => {
                if self.state.is_upper_bound(fitness, self.player) {
                    Some(action)
                } else if self.state.is_lower_bound(fitness, self.player) {
                    if self
                        .losing_action
                        .as_ref()
                        .map_or(true, |act| act.path.len() < action.path.len())
                    {
                        let act = self.losing_action.replace(action);
                        act.map(|act| self.discard_path(act.path));
                    }
                    None
                } else {
                    self.add_terminated(action);
                    None
                }
            }
            Ok(MiniMax::Terminated(mut path, Branch::Equal(fitness))) => {
                path.push(action.path.pop().unwrap());
                self.discard_path(action.path);
                let action = Action { fitness, path };
                if self.state.is_upper_bound(fitness, self.player) {
                    Some(action)
                } else if self.state.is_lower_bound(fitness, self.player) {
                    if self
                        .losing_action
                        .as_ref()
                        .map_or(true, |act| act.path.len() < action.path.len())
                    {
                        let act = self.losing_action.replace(action);
                        act.map(|act| self.discard_path(act.path));
                    }
                    None
                } else {
                    self.add_terminated(action);
                    None
                }
            }
            Ok(MiniMax::Terminated(mut path, Branch::Worse(fitness))) => {
                path.push(action.path.pop().unwrap());
                self.discard_path(action.path);
                let action = Action { fitness, path };
                self.add_partially_terminated(action);
                None
            }
            Ok(MiniMax::Open(mut path, Branch::Worse(fitness))) => {
                path.push(action.path.pop().unwrap());
                self.discard_path(action.path);
                let action = Action { fitness, path };
                self.unfinished.push(action);
                None
            }
            Ok(MiniMax::Open(mut path, Branch::Equal(fitness))) => {
                path.push(action.path.pop().unwrap());
                self.discard_path(action.path);
                let action = Action { fitness, path };
                self.add_best(action);
                None
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

        // Sort the actions so the most probable one is checked first.
        // This allows for faster cutoffs. Note that depending on the fitness
        // function, this can hit some fairly bad cases.
        if active {
            game_states.sort_by(|(_, _, a), (_, _, b)| b.cmp(a));
        } else {
            game_states.sort_by(|(_, _, a), (_, _, b)| a.cmp(b));
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
        &mut self,
        mut path: impl Iterator<Item = T::Action>,
        game_state: T,
        depth: u32,
        alpha: Option<T::Fitness>,
        beta: Option<T::Fitness>,
        condition: &mut U,
    ) -> Result<MiniMax<T>, CancelledError> {
        if !condition.step() {
            return Err(CancelledError);
        }

        let action = if let Some(action) = path.next() {
            action
        } else {
            return self.minimax(game_state, depth, alpha, beta, condition);
        };

        if depth == 0 {
            unreachable!("lowest depth with non empty path");
        }

        let (active, mut game_states) = self.generate_game_states(&game_state);

        let mut state = State::new(
            self.new_path(),
            game_state,
            self.player,
            alpha,
            None,
            active,
        );
        match game_states.iter().position(|(_, a, _)| *a == action) {
            Some(idx) => {
                let (game_state, action, fitness) = game_states.remove(idx);

                let minimax = self
                    .minimax_with_path(
                        path,
                        game_state,
                        depth - 1,
                        state.alpha,
                        state.beta,
                        condition,
                    )?
                    .with(self, action, fitness);

                if let Some(cutoff) = state.bind(self, minimax) {
                    return Ok(cutoff);
                }
            }
            None => unreachable!("path segment not found"),
        }

        for (game_state, action, fitness) in game_states {
            let minimax = self
                .minimax(game_state, depth - 1, state.alpha, state.beta, condition)?
                .with(self, action, fitness);
            if let Some(cutoff) = state.bind(self, minimax) {
                return Ok(cutoff);
            }
        }

        Ok(state.consume())
    }

    fn minimax<U: RunCondition>(
        &mut self,
        game_state: T,
        depth: u32,
        alpha: Option<T::Fitness>,
        beta: Option<T::Fitness>,
        condition: &mut U,
    ) -> Result<MiniMax<T>, CancelledError> {
        if !condition.step() {
            return Err(CancelledError);
        }

        if depth == 0 {
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

            return Ok(selected.map_or(MiniMax::DeadEnd, |(action, fitness)| {
                let mut path = self.new_path();
                path.push(action);
                MiniMax::Open(path, Branch::Equal(fitness))
            }));
        }

        let (active, game_states) = self.generate_game_states(&game_state);

        if game_states.is_empty() {
            return Ok(MiniMax::DeadEnd);
        }

        let mut state = State::new(
            self.new_path(),
            game_state,
            self.player,
            alpha,
            beta,
            active,
        );
        for (game_state, action, fitness) in game_states {
            let minimax = self
                .minimax(game_state, depth - 1, state.alpha, state.beta, condition)?
                .with(self, action, fitness);
            if let Some(cutoff) = state.bind(self, minimax) {
                return Ok(cutoff);
            }
        }

        Ok(state.consume())
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

impl<T: Game> State<T> {
    fn new(
        path: Vec<T::Action>,
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
            path,
            terminated: true,
            active,
        }
    }

    fn update_best_action(
        &mut self,
        ctxt: &mut Ctxt<'_, T>,
        path: Vec<T::Action>,
        fitness: Branch<T>,
    ) {
        assert!(!path.is_empty());
        ctxt.discard_path(mem::replace(&mut self.path, path));
        self.best_fitness = Some(fitness);
    }

    fn bind(&mut self, ctxt: &mut Ctxt<'_, T>, value: MiniMax<T>) -> Option<MiniMax<T>> {
        match value {
            MiniMax::DeadEnd => unreachable!(),
            MiniMax::Terminated(path, Branch::Equal(fitness)) => {
                self.bind_equal(ctxt, path, fitness, true);
            }
            MiniMax::Terminated(path, Branch::Better(fitness)) => {
                self.bind_better(ctxt, path, fitness, true);
            }
            MiniMax::Terminated(path, Branch::Worse(fitness)) => {
                self.bind_worse(ctxt, path, fitness, true);
            }
            MiniMax::Open(path, Branch::Equal(fitness)) => {
                self.bind_equal(ctxt, path, fitness, false);
            }
            MiniMax::Open(path, Branch::Better(fitness)) => {
                self.bind_better(ctxt, path, fitness, false);
            }
            MiniMax::Open(path, Branch::Worse(fitness)) => {
                self.bind_worse(ctxt, path, fitness, false);
            }
        }

        let branch = match self.best_fitness {
            Some(Branch::Equal(fitness)) | Some(Branch::Better(fitness))
                if self.active && self.state.is_upper_bound(fitness, self.player) =>
            {
                Branch::Equal(fitness)
            }
            Some(Branch::Equal(fitness)) | Some(Branch::Worse(fitness))
                if !self.active && self.state.is_lower_bound(fitness, self.player) =>
            {
                Branch::Equal(fitness)
            }
            _ => match (self.alpha, self.beta) {
                (Some(alpha), Some(beta)) if alpha >= beta => {
                    if self.active {
                        Branch::Better(self.alpha.unwrap())
                    } else {
                        Branch::Worse(self.beta.unwrap())
                    }
                }
                _ => return None,
            },
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

    fn bind_equal(
        &mut self,
        ctxt: &mut Ctxt<'_, T>,
        path: Vec<T::Action>,
        fitness: T::Fitness,
        terminated: bool,
    ) {
        self.terminated &= terminated;
        if self.active {
            if terminated && self.state.is_upper_bound(fitness, self.player) {
                self.update_best_action(ctxt, path, Branch::Equal(fitness));
                self.terminated = true;
            } else {
                self.alpha = Some(self.alpha.map_or(fitness, |value| cmp::max(value, fitness)));
                if self
                    .best_fitness
                    .as_ref()
                    .map_or(true, |old| old.fitness() <= fitness)
                {
                    self.update_best_action(ctxt, path, Branch::Equal(fitness));
                } else {
                    ctxt.discard_path(path);
                }
            }
        } else if terminated && self.state.is_lower_bound(fitness, self.player) {
            self.update_best_action(ctxt, path, Branch::Equal(fitness));
            self.terminated = true;
        } else {
            self.beta = Some(self.beta.map_or(fitness, |value| cmp::min(value, fitness)));
            if self
                .best_fitness
                .as_ref()
                .map_or(true, |old| old.fitness() >= fitness)
            {
                self.update_best_action(ctxt, path, Branch::Equal(fitness));
            } else {
                ctxt.discard_path(path);
            }
        }
    }

    fn bind_better(
        &mut self,
        ctxt: &mut Ctxt<'_, T>,
        path: Vec<T::Action>,
        fitness: T::Fitness,
        terminated: bool,
    ) {
        self.terminated &= terminated;
        if self.active {
            debug_assert!(self.alpha.map_or(true, |value| value <= fitness));
            debug_assert!(self
                .best_fitness
                .as_ref()
                .map_or(true, |value| value.fitness() <= fitness));

            self.alpha = Some(fitness);
            self.update_best_action(ctxt, path, Branch::Better(fitness));
        } else if self
            .best_fitness
            .as_ref()
            .map_or(true, |old| old.fitness() > fitness)
        {
            self.update_best_action(ctxt, path, Branch::Better(fitness));
        } else {
            ctxt.discard_path(path);
        }
    }

    fn bind_worse(
        &mut self,
        ctxt: &mut Ctxt<'_, T>,
        path: Vec<T::Action>,
        fitness: T::Fitness,
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
            self.update_best_action(ctxt, path, Branch::Worse(fitness));
        } else if self
            .best_fitness
            .as_ref()
            .map_or(true, |old| old.fitness() < fitness)
        {
            self.update_best_action(ctxt, path, Branch::Worse(fitness));
        } else {
            ctxt.discard_path(path);
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
