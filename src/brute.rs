use crate::{Game, GameBot};

use std::time::{Duration, Instant};

pub struct Bot<T: Game> {
    player: T::Player,
    calls: u32,
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

        let mut selected = 0;
        for depth in 0.. {
            let mut actions = actions.iter().enumerate();

            let mut best = {
                let action = actions.next()?;
                let value = self.minimax(state, &action.1, depth);
                (action, value)
            };

            for action in actions {
                let new = self.minimax(state, &action.1, depth);
                if new > best.1 {
                    best = (action, new);
                }
            }

            if now.elapsed() > duration {
                selected = (best.0).0;
                break;
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

    fn minimax(&mut self, state: &T, action: &T::Action, depth: u32) -> T::Fitness {
        self.calls += 1;

        if depth == 0 {
            state.look_ahead(&action, &self.player)
        }
        else {
            let mut state = state.clone();
            let fitness = state.execute(&action, &self.player);
            let (active, actions) = state.actions(&self.player);
            
            let iter = actions.into_iter().map(|action| {
                self.minimax(&state, &action, depth - 1)
            });

            if active { 
                iter.max()
            } else { 
                iter.min()
            }.unwrap_or(fitness)
        }
    }
}