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

fn cutoff<T: Ord>(alpha: Option<T>, beta: Option<T>) -> bool {
    alpha.map_or(false, |alpha| beta.map_or(false, |beta| alpha >= beta))
}

impl<T: Game> Bot<T> {
    pub fn new(player: T::Player) -> Self {
        Self { player }
    }

    fn minimax(
        &mut self,
        state: T,
        depth: u32,
        mut alpha: Option<T::Fitness>,
        mut beta: Option<T::Fitness>,
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
                    (state, fitness)
                })
                .collect();

            if states.is_empty() {
                return Ok(MiniMax::DeadEnd);
            }

            states.sort_unstable_by_key(|(_, fitness)| *fitness);

            let mut terminated = true;
            let mut result = None;
            for (state, fitness) in states.into_iter().rev() {
                match self.minimax(state, depth - 1, alpha, beta, end_time)? {
                    MiniMax::DeadEnd => {
                        if active {
                            alpha = Some(alpha.map_or(fitness, |value| cmp::max(value, fitness)));
                            result = Some(result.map_or(fitness, |value| cmp::max(value, fitness)));
                        } else {
                            beta = Some(beta.map_or(fitness, |value| cmp::min(value, fitness)));
                            result = Some(result.map_or(fitness, |value| cmp::min(value, fitness)));
                        }
                    }
                    MiniMax::Terminated(Branch::Equal(fitness)) => {
                        if active {
                            alpha = Some(alpha.map_or(fitness, |value| cmp::max(value, fitness)));
                            result = Some(result.map_or(fitness, |value| cmp::max(value, fitness)));
                        } else {
                            beta = Some(beta.map_or(fitness, |value| cmp::min(value, fitness)));
                            result = Some(result.map_or(fitness, |value| cmp::min(value, fitness)));
                        }
                    }
                    MiniMax::Terminated(Branch::Better(fitness)) => {
                        if active {
                            assert!(alpha.map_or(true, |value| value <= fitness));
                            alpha = Some(fitness);
                            assert!(result.map_or(true, |value| value <= fitness));
                            result = Some(result.map_or(fitness, |value| cmp::max(value, fitness)));
                        } else {
                            result = Some(result.map_or(fitness, |value| cmp::min(value, fitness)));
                        }
                    }
                    MiniMax::Terminated(Branch::Worse(fitness)) => {
                        if active {
                            result = Some(result.map_or(fitness, |value| cmp::max(value, fitness)));
                        } else {
                            assert!(beta.map_or(true, |value| value >= fitness));
                            beta = Some(fitness);
                            assert!(result.map_or(true, |value| value >= fitness));
                            result = Some(result.map_or(fitness, |value| cmp::min(value, fitness)));
                        }
                    }
                    MiniMax::Open(Branch::Equal(fitness)) => {
                        terminated = false;

                        if active {
                            alpha = Some(alpha.map_or(fitness, |value| cmp::max(value, fitness)));
                            result = Some(result.map_or(fitness, |value| cmp::max(value, fitness)));
                        } else {
                            beta = Some(beta.map_or(fitness, |value| cmp::min(value, fitness)));
                            result = Some(result.map_or(fitness, |value| cmp::min(value, fitness)));
                        }
                    }
                    MiniMax::Open(Branch::Better(fitness)) => {
                        terminated = false;
                        if active {
                            assert!(alpha.map_or(true, |value| value <= fitness));
                            alpha = Some(fitness);
                            assert!(result.map_or(true, |value| value <= fitness));
                            result = Some(result.map_or(fitness, |value| cmp::max(value, fitness)));
                        } else {
                            result = Some(result.map_or(fitness, |value| cmp::min(value, fitness)));
                        }
                    }
                    MiniMax::Open(Branch::Worse(fitness)) => {
                        terminated = false;
                        if active {
                            result = Some(result.map_or(fitness, |value| cmp::max(value, fitness)));
                        } else {
                            assert!(beta.map_or(true, |value| value >= fitness));
                            beta = Some(fitness);
                            assert!(result.map_or(true, |value| value >= fitness));
                            result = Some(result.map_or(fitness, |value| cmp::min(value, fitness)));
                        }
                    }
                }

                if cutoff(alpha, beta) {
                    break;
                }
            }

            let branch = match (cutoff(alpha, beta), active) {
                (true, true) => Branch::Better(alpha.unwrap()),
                (true, false) => Branch::Worse(beta.unwrap()),
                (false, _) => Branch::Equal(result.unwrap()),
            };

            if terminated {
                Ok(MiniMax::Terminated(branch))
            } else {
                Ok(MiniMax::Open(branch))
            }
        }
    }
}
