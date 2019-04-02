use crate::{Game, GameBot};

use std::cmp::Ord;
use std::time::{Duration, Instant};

pub struct Bot<T: Game> {
    player: T::Player,
    calls: u32
}

struct Meta<'a, T: Game> {
    depth: u32,
    alpha: Option<T::Fitness>,
    beta: Option<T::Fitness>,
    discovered: &'a mut bool,
}

fn select<T: Ord>(old: &mut Option<T>, new: T, max: bool) {
    *old = match old.take() {
        Some(old) => if max {
            Some(std::cmp::max(old, new))
        }
        else {
            Some(std::cmp::min(old, new))
        },
        None => Some(new)
    }
}

impl<T: Game> GameBot<T> for Bot<T> {
    fn select(&mut self, state: &T, duration: Duration) -> Option<T::Action> {
        let now = Instant::now();

        let (active, actions) = state.actions(&self.player);
        if !active { return None }

        let mut actions: Vec<_> = actions.into_iter().collect();
        if actions.len() < 2 {
            return actions.pop()
        }

        actions.sort_by_cached_key(|action| {
            state.look_ahead(action, &self.player)
        });

        let mut selected = 0;
        for depth in 0.. {
            let mut actions_iter = actions.iter().enumerate().rev();
            let mut discovered = false;

            let mut best = {
                let action = actions_iter.next().unwrap();
                let mut state = state.clone();
                let fitness = state.execute(&action.1, &self.player);
                let value = self.minimax(state, Meta { depth, alpha: None, beta: None, discovered: &mut discovered}).unwrap_or(fitness);
                (action, value)
            };

            for action in actions_iter {
                let mut state = state.clone();
                let fitness = state.execute(&action.1, &self.player);
                let new = self.minimax(state, Meta { depth, alpha: Some(best.1), beta: None, discovered: &mut discovered}).unwrap_or(fitness);
                
                if new > best.1 {
                    best = (action, new);
                }
            }
            
            selected = (best.0).0;
            if now.elapsed() > duration || !discovered {
                println!("Maximum depth: {}", depth);
                break;
            }
            else {
                actions.swap(0, selected);
            }
        }
        Some(actions.swap_remove(selected))
    }
}

impl<T: Game> Bot<T> {
    pub fn new(player: T::Player) -> Self {
        Self {
            player,
            calls: 0
        }
    }

    pub fn calls(&self) -> u32 {
        self.calls
    }

    fn minimax(&mut self, state: T, mut meta: Meta<T>) -> Option<T::Fitness> {
        self.calls += 1;
        if meta.depth == 0 {
            *meta.discovered = true;
            None
        }
        else {
            let (active, actions) = state.actions(&self.player);
            let mut states: Vec<(T, T::Fitness)> = actions.into_iter().map(|action| {
                let mut state = state.clone();
                let fitness = state.execute(&action, &self.player);
                (state, fitness)
            }).collect();
            states.sort_unstable_by_key(|(_, fitness)| *fitness);
            
            let mut res = None;
            for (state, fitness) in states.into_iter().rev() {
                let v = self.minimax(state, Meta { depth: meta.depth - 1, discovered: &mut *meta.discovered, ..meta }).unwrap_or(fitness);
                
                if active { select(&mut meta.alpha, v, true) }
                else { select(&mut meta.beta, v, false) }
                select(&mut res, v, active);
                if meta.alpha >= meta.beta {
                    break;
                }
            }
            
            res
        }
    }
}