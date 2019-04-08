use crate::{Game, GameBot};

use std::cmp::{self, Ord, Ordering};
use std::mem;
use std::time::{Duration, Instant};

struct OutOfTimeError;

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
    active: bool
}

impl<T: Game> State<T> {
    fn new(alpha: Option<T::Fitness>, beta: Option<T::Fitness>, active: bool) -> Self {
        Self {
            alpha,
            beta,
            best_fitness: None,
            path: Vec::new(),
            terminated: true,
            active
        }
    }


    fn update_best_action(&mut self, path: Vec<T::Action>, action: T::Action, fitness: T::Fitness) {
        self.path = path;
        self.path.push(action);
        self.best_fitness = Some(fitness)
    }

    fn bind_equal(&mut self, path: Vec<T::Action>, fitness: T::Fitness, action: T::Action, terminated: bool) {
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

    fn bind_better(&mut self, path: Vec<T::Action>, fitness: T::Fitness, action: T::Action, terminated: bool) {
        self.terminated &= terminated;
        if self.active {
            debug_assert!(self.alpha.map_or(true, |value| value <= fitness));
            self.alpha = Some(fitness);
            debug_assert!(self.best_fitness.map_or(true, |value| value <= fitness));
            self.update_best_action(path, action, fitness);

        }
    }

    fn bind_worse(&mut self, path: Vec<T::Action>, fitness: T::Fitness, action: T::Action, terminated: bool) {
        self.terminated &= terminated;
        if !self.active {
            debug_assert!(self.beta.map_or(true, |value| value >= fitness));
            self.beta = Some(fitness);
            debug_assert!(self.best_fitness.map_or(true, |value| value >= fitness));
            self.update_best_action(path, action, fitness);
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


    fn consume(self) -> MiniMax<T> {
        let branch = match (self.is_cutoff(), self.active) {
            (true, true) => Branch::Better(self.alpha.unwrap()),
            (true, false) => Branch::Worse(self.beta.unwrap()),
            (false, _) => self.best_fitness.map(|res| Branch::Equal(res))
            .unwrap_or_else(|| {
                if self.active { Branch::Worse(self.alpha.unwrap()) }
                else { Branch::Better(self.beta.unwrap()) }
            }),
        };

        if self.terminated {
            MiniMax::Terminated(self.path, branch)
        } else {
            MiniMax::Open(self.path, branch)
        }
    }
}

pub struct Bot<T: Game> {
    player: T::Player,
}

impl<T: Game> GameBot<T> for Bot<T> {
    fn select(&mut self, state: &T, duration: Duration) -> Option<T::Action> {
        let end_time = Instant::now() + duration;

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
        let mut terminated: Option<(T::Action, T::Fitness)> = None;
        let mut best_path = Vec::new();
        let mut best_fitness: Option<T::Fitness> = None;
        for depth in 1.. {
            let mut prev_best_path = mem::replace(&mut best_path, Vec::new());
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
                    mem::replace(&mut prev_best_path, Vec::new()),
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
                    Ok(MiniMax::Terminated(_path, Branch::Equal(fitness))) => {
                        if let Ordering::Less = terminated
                            .as_ref()
                            .map_or(Ordering::Less, |(_action, best_term)| {
                                best_term.cmp(&fitness)
                            })
                        {
                            terminated = Some((action, fitness));
                        }
                    }
                    Ok(MiniMax::Terminated(_path, Branch::Worse(_)))
                    | Ok(MiniMax::Open(_path, Branch::Worse(_))) => {
                        // keep the best action on top of the stack at all times
                        let len = actions.len();
                        actions.insert(len - 1, action);
                    }
                    Ok(MiniMax::Open(path, Branch::Equal(fitness))) => {
                        actions.push(action);
                        best_fitness = Some(fitness);
                        best_path = path;
                    }
                    Ok(MiniMax::Terminated(_path,Branch::Better(_)))
                    | Ok(MiniMax::Open(_path, Branch::Better(_))) => {
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
            if best_fitness.map_or(true, |best_fitness| best_fitness <= terminated_fitness) {
                Some(terminated_action)
            } else {
                assert!(!actions.is_empty());
                actions.pop()
            }
        } else {
            assert!(!actions.is_empty());
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
        mut path: Vec<T::Action>,
        game_state: T,
        depth: u32,
        alpha: Option<T::Fitness>,
        beta: Option<T::Fitness>,
        end_time: Instant,
    ) -> Result<MiniMax<T>, OutOfTimeError> {
        if Instant::now() > end_time {
            Err(OutOfTimeError)
        } else if depth == 0 {
            debug_assert!(path.is_empty(), "The previous search should not have reached this deep");

            let (active, actions) = game_state.actions(&self.player);
            let selected = if active {
                actions
                    .into_iter()
                    .map(|action| {
                        let fitness = game_state.look_ahead(&action, &self.player);
                        (action, fitness)
                    }).max_by_key(|(_, fitness)| *fitness)
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
                .map(|action| {
                    let mut game_state = game_state.clone();
                    let fitness = game_state.execute(&action, &self.player);
                    (game_state, action, fitness)
                })
                .collect();

            if states.is_empty() {
                return Ok(MiniMax::DeadEnd);
            }

            let mut state = State::new(alpha, beta, active);

            states.sort_unstable_by_key(|(_, _,fitness)| *fitness);
            path.pop().map(|action| {
                let mut game_state = game_state.clone();
                let fitness = game_state.execute(&action, &self.player);
                states.push((game_state, action, fitness))
            });
            for (game_state, action, fitness) in states.into_iter().rev() {
                match self.minimax(mem::replace(&mut path, Vec::new()), game_state, depth - 1, alpha, beta, end_time)? {
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
