use crate::{Game, GameBot};

pub struct Bot<T: Game> {
    player: T::Player,
    depth: u8,
    calls: u32
}

impl<T: Game> GameBot<T> for Bot<T> {
    fn select(&mut self, state: &T) -> Option<T::Action> {
        let (active, actions) = state.actions(&self.player);
        if !active {
            None
        }
        else {
            let mut actions = actions.into_iter();

            let mut best = {
                let action = actions.next()?;
                let value = self.minimax(state, &action, self.depth);
                (action, value)
            };

            for action in actions {
                let new = self.minimax(state, &action, self.depth);
                if new > best.1 {
                    best = (action, new);
                }
            }
            Some(best.0)
        }
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

    fn minimax(&mut self, state: &T, action: &T::Action, depth: u8) -> T::Fitness {
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