use crate::{Game, GameBot};

use std::cmp::Ord;

pub struct Bot<T: Game> {
    player: T::Player,

    depth: u8,
    calls: u32
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
    fn select(&mut self, state: &T) -> Option<T::Action> {
        let (active, actions) = state.actions(&self.player);
        if !active { return None }

        let mut actions: Vec<_> = actions.into_iter().collect();
        actions.sort_by_cached_key(|action| {
            state.look_ahead(action, &self.player)
        });
        let mut actions = actions.into_iter().rev();

        let mut best = {
            let action = actions.next()?;
            let mut state = state.clone();
            let fitness = state.execute(&action, &self.player);
            let value = self.minimax(state, self.depth, None, None).unwrap_or(fitness);
            (action, value)
        };

        for action in actions {
            let mut state = state.clone();
            let fitness = state.execute(&action, &self.player);
            let new = self.minimax(state, self.depth, Some(best.1), None).unwrap_or(fitness);
            
            if new > best.1 {
                best = (action, new);
            }
        }
        Some(best.0)
    }
}

impl<T: Game> Bot<T> {
    pub fn new(player: T::Player, depth: u8) -> Self {
        Self {
            player,

            depth,
            calls: 0
        }
    }

    pub fn calls(&self) -> u32 {
        self.calls
    }

    fn minimax(&mut self, state: T, depth: u8, mut alpha: Option<T::Fitness>, mut beta: Option<T::Fitness>) -> Option<T::Fitness> {
        self.calls += 1;
        if depth == 0 {
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
                let v = self.minimax(state, depth - 1, alpha, beta).unwrap_or(fitness);
                
                if active { select(&mut alpha, v, true) }
                else { select(&mut beta, v, false) }
                select(&mut res, v, active);
                if alpha >= beta {
                    break;
                }
            }
            
            res
        }
    }
}