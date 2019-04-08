use crate::{Game, GameBot};

use std::cmp::{self, Ord, Ordering};
use std::mem;
use std::time::{Duration, Instant};

pub struct Bot<T: Game> {
    player: T::Player,
}

struct OutOfTimeError;

enum MiniMax<T: Game> {
    /// No new elements were found in this branch
    Terminated(Branch<T>),
    /// New elements were found
    Open(Branch<T>),
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
    best_action: Option<(T::Action, T::Fitness)>,
    terminated: bool,
    active: bool
}

impl<T: Game> State<T> {
    fn new(alpha: Option<T::Fitness>, beta: Option<T::Fitness>, active: bool) -> Self {
        Self {
            alpha,
            beta,
            best_action: None,
            terminated: true,
            active
        }
    }

    fn bind_equal(&mut self, fitness: T::Fitness, action: T::Action, terminated: bool) {
        self.terminated &= terminated;
        if self.active {
            self.alpha = Some(self.alpha.map_or(fitness, |value| cmp::max(value, fitness)));
            self.best_action = Some(if let Some((current_action, current_fitness)) = self.best_action.take() {
                if current_fitness > fitness {
                    (current_action, current_fitness)
                }
                else { (action, fitness) }
            }else { (action, fitness) })
        } else {
            self.beta = Some(self.beta.map_or(fitness, |value| cmp::min(value, fitness)));
            self.best_action = Some(if let Some((current_action, current_fitness)) = self.best_action.take() {
                if current_fitness < fitness {
                    (current_action, current_fitness)
                }
                else { (action, fitness) }
            } else { (action, fitness) })
        }
    }

    fn bind_better(&mut self, fitness: T::Fitness, action: T::Action, terminated: bool) {
        self.terminated &= terminated;
        if self.active {
            debug_assert!(self.alpha.map_or(true, |value| value <= fitness));
            self.alpha = Some(fitness);
            debug_assert!(self.best_action.as_ref().map_or(true, |value| value.1 <= fitness));
            self.best_action = Some((action, fitness));
        }
    }

    fn bind_worse(&mut self, fitness: T::Fitness, action: T::Action, terminated: bool) {
        self.terminated &= terminated;
        if !self.active {
            debug_assert!(self.beta.map_or(true, |value| value >= fitness));
            self.beta = Some(fitness);
            debug_assert!(self.best_action.as_ref().map_or(true, |value| value.1 >= fitness));
            self.best_action = Some((action, fitness));
        }
    }

    fn is_cutoff(&self) -> bool {
        if let (Some(ref alpha), Some(ref beta)) = (self.alpha, self.beta) {
            alpha >= beta 
        }
        else {
            false
        }
    }


    fn consume(mut self) -> MiniMax<T> {
        let branch = match (self.is_cutoff(), self.active) {
            (true, true) => Branch::Better(self.alpha.unwrap()),
            (true, false) => Branch::Worse(self.beta.unwrap()),
            (false, _) => self.best_action.take().map(|res| Branch::Equal(res.1))
            .unwrap_or_else(|| {
                if self.active { Branch::Worse(self.alpha.unwrap()) }
                else { Branch::Better(self.beta.unwrap()) }
            }),
        };

        if self.terminated {
            MiniMax::Terminated(branch)
        } else {
            MiniMax::Open(branch)
        }
    }
}

impl<T: Game> GameBot<T> for Bot<T> {
    fn select(&mut self, state: &T, duration: Duration) -> Option<T::Action> {
        let end_time = Instant::now() + duration;

        let (active, actions) = state.actions(&self.player);
        if !active {
            return None;
        }

        let mut actions = actions.into_iter().collect::<Vec<_>>();
        if actions.is_empty() {
            return None;
        }
        actions.sort_by_cached_key(|a| state.look_ahead(&a, &self.player));

        // the best action which already terminated
        let mut terminated: Option<(T::Action, T::Fitness)> = None;
        let mut best_fitness: Option<T::Fitness> = None;
        for depth in 1.. {
            for action in mem::replace(&mut actions, Vec::new()).into_iter().rev() {
                if Instant::now() > end_time {
                    if actions.is_empty() {
                        actions.push(action)
                    }
                    break;
                }

                let mut state = state.clone();
                let fitness = state.execute(&action, &self.player);
                match self.minimax(
                    state,
                    depth - 1,
                    best_fitness.filter(|_| !actions.is_empty()),
                    None,
                    end_time,
                ) {
                    // if the time is over, return the current best element or the best element of the previous depth
                    Err(OutOfTimeError) => {
                        if actions.is_empty() {
                            actions.push(action)
                        }
                        break;
                    }
                    Ok(MiniMax::DeadEnd) => {
                        if let Ordering::Less = terminated
                            .as_ref()
                            .map_or(Ordering::Less, |(_action, best_term)| {
                                best_term.cmp(&fitness)
                            })
                        {
                            terminated = Some((action, fitness));
                        }
                    }
                    Ok(MiniMax::Terminated(Branch::Equal(fitness))) => {
                        if let Ordering::Less = terminated
                            .as_ref()
                            .map_or(Ordering::Less, |(_action, best_term)| {
                                best_term.cmp(&fitness)
                            })
                        {
                            terminated = Some((action, fitness));
                        }
                    }
                    Ok(MiniMax::Terminated(Branch::Worse(_)))
                    | Ok(MiniMax::Open(Branch::Worse(_))) => {
                        assert!(!actions.is_empty());
                        let len = actions.len();
                        actions.insert(len - 1, action);
                    }
                    Ok(MiniMax::Open(Branch::Equal(fitness))) => {
                        actions.push(action);
                        best_fitness = Some(fitness);
                    }
                    Ok(MiniMax::Terminated(Branch::Better(_)))
                    | Ok(MiniMax::Open(Branch::Better(_))) => {
                        unreachable!("beta cutoff at highest depth")
                    }
                }
            }

            if Instant::now() > end_time || actions.is_empty() {
                best_fitness = None;
                break;
            }
        }

        if let Some((terminated_action, terminated_fitness)) = terminated {
            if let Ordering::Less = best_fitness.map_or(Ordering::Less, |best_fitness| {
                best_fitness.cmp(&terminated_fitness)
            }) {
                Some(terminated_action)
            } else {
                assert!(!actions.is_empty());
                actions.pop()
            }
        } else {
            actions.pop()
        }
    }
}

impl<T: Game> Bot<T> {
    pub fn new(player: T::Player) -> Self {
        Self { player }
    }

    fn minimax(
        &mut self,
        state: T,
        depth: u32,
        alpha: Option<T::Fitness>,
        beta: Option<T::Fitness>,
        end_time: Instant,
    ) -> Result<MiniMax<T>, OutOfTimeError> {
        if Instant::now() > end_time {
            Err(OutOfTimeError)
        } else if depth == 0 {
            let (active, actions) = state.actions(&self.player);
            let selected = if active {
                actions
                    .into_iter()
                    .map(|action| state.look_ahead(&action, &self.player))
                    .max()
            } else {
                actions
                    .into_iter()
                    .map(|action| state.look_ahead(&action, &self.player))
                    .min()
            };

            Ok(selected
                .map(|fitness| MiniMax::Open(Branch::Equal(fitness)))
                .unwrap_or(MiniMax::DeadEnd))
        } else {
            let (active, actions) = state.actions(&self.player);
            let mut states: Vec<_> = actions
                .into_iter()
                .map(|action| {
                    let mut state = state.clone();
                    let fitness = state.execute(&action, &self.player);
                    (state, action, fitness)
                })
                .collect();

            if states.is_empty() {
                return Ok(MiniMax::DeadEnd);
            }

            let mut state = State::new(alpha, beta, active);

            states.sort_unstable_by_key(|(_, _,fitness)| *fitness);
            for (game_state, action, fitness) in states.into_iter().rev() {
                match self.minimax(game_state, depth - 1, alpha, beta, end_time)? {
                    MiniMax::DeadEnd => {
                        state.bind_equal(fitness, action, true);
                    }
                    MiniMax::Terminated(Branch::Equal(fitness)) => {
                        state.bind_equal(fitness, action, true);
                    }
                    MiniMax::Terminated(Branch::Better(fitness)) => {
                        state.bind_better(fitness, action, true);
                    }
                    MiniMax::Terminated(Branch::Worse(fitness)) => {
                        state.bind_worse(fitness, action, true);
                    }
                    MiniMax::Open(Branch::Equal(fitness)) => {
                        state.bind_equal(fitness, action, false);
                    }
                    MiniMax::Open(Branch::Better(fitness)) => {
                        state.bind_better(fitness, action, false);
                    }
                    MiniMax::Open(Branch::Worse(fitness)) => {
                        state.bind_worse(fitness, action, false);
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
