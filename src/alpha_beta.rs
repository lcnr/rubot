use crate::{Game, GameBot};

use std::cmp::{Ord, Ordering};
use std::time::{Duration, Instant};

pub struct Bot<T: Game> {
    player: T::Player,
}

struct Meta {
    start: Instant,
    duration: Duration,
    discovered: bool,
}

impl<T: Game> GameBot<T> for Bot<T> {
    fn select(&mut self, state: &T, duration: Duration) -> Option<T::Action> {
        let start = Instant::now();

        let (active, actions) = state.actions(&self.player);
        if !active { return None }

        let mut actions = actions.into_iter().collect::<Vec<_>>();
        actions.sort_by_cached_key(|a| state.look_ahead(&a, &self.player));
        let last = actions.len() - 1;
        
        for depth in 1.. {
            let mut discovered = false;
            let mut best_fitness = None;
            let mut best_index = last;
            for (idx, ref action) in actions.iter_mut().enumerate().rev() {
                if start.elapsed() > duration {
                    break;
                }

                let mut meta = Meta {
                    start,
                    duration,
                    discovered: false
                };
                

                let mut state = state.clone();
                let fitness = state.execute(&action, &self.player);
                let fitness = self.minimax(state, depth, best_fitness, None, &mut meta).unwrap_or(fitness);
                if meta.discovered {
                    discovered = true;

                    if let Ordering::Less = best_fitness.map_or(Ordering::Less, |best| best.cmp(&fitness)) {
                        best_fitness = Some(fitness);
                        best_index = idx;
                    }
                }
            }
            actions.swap(best_index, last);

            println!("{:?}", actions);

            if start.elapsed() > duration || !discovered {
                println!("Maximum depth: {}", depth);
                break;
            }
        }
        actions.pop()
    }
}

impl<T: Game> Bot<T> {
    pub fn new(player: T::Player) -> Self {
        Self {
            player
        }
    }
    fn minimax(&mut self, state: T, depth: u32, mut alpha: Option<T::Fitness>, mut beta: Option<T::Fitness>, meta: &mut Meta) -> Option<T::Fitness> {
        if depth == 0 {
            meta.discovered = true;
            None
        }
        else if meta.start.elapsed() > meta.duration {
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
            for (state, mut fitness) in states.into_iter().rev() {
                fitness = self.minimax(state, depth - 1, alpha, beta, &mut *meta).unwrap_or(fitness);
                
                if active { 
                    if let Ordering::Less = alpha.map_or(Ordering::Less, |alpha| alpha.cmp(&fitness)) {
                        alpha = Some(fitness);
                    }

                    if let Ordering::Less = res.map_or(Ordering::Less, |res: T::Fitness| res.cmp(&fitness)) {
                        res = Some(fitness)
                    }
                }
                else {
                    if let Ordering::Greater = beta.map_or(Ordering::Greater, |beta| beta.cmp(&fitness)) {
                        beta = Some(fitness);
                    }

                    if let Ordering::Greater = res.map_or(Ordering::Greater, |res| res.cmp(&fitness)) {
                        res = Some(fitness)
                    }
                }

                if alpha >= beta {
                    break;
                }
            }
            
            res
        }
    }
}