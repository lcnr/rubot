//! This module contains a bot which simply brute forces every possible action, this bot should only be used for testing.
use crate::Game;

use std::cmp::{self, Ordering};
use std::fmt;

/// A bot which uses brute force to calculate the optimal move
pub struct Brute<T: Game> {
    player: T::Player,
}

impl<T: Game> Brute<T> {
    pub fn new(player: T::Player) -> Self {
        Self { player }
    }

    pub fn select(&mut self, state: &T, depth: u32) -> Option<T::Action> {
        let (active, actions) = state.actions(self.player);
        if !active {
            return None;
        }

        let mut actions = actions.into_iter();

        let mut best = {
            let action = actions.next()?;
            let value = self.minimax(state, &action, depth);
            (action, value)
        };

        for action in actions {
            let new = self.minimax(state, &action, depth);
            if new > best.1 {
                best = (action, new);
            }
        }

        Some(best.0)
    }

    pub fn check_if_best(&mut self, state: &T, best: Option<&T::Action>, depth: u32) -> bool {
        let (active, actions) = state.actions(self.player);
        if !active {
            return best.is_none();
        }

        let mut actions = actions.into_iter();
        if best.is_none() {
            return actions.next().is_none();
        }

        let mut best = self.minimax(state, best.unwrap(), depth);

        for action in actions {
            let new = self.minimax(state, &action, depth);
            if new > best {
                return false;
            }
        }

        true
    }

    /// lists all actions with a fitness at `completed_depth + 1` which is better than the worst action
    /// of all best actions at `completed_depth`
    pub fn allowed_actions(&mut self, state: &T, completed_depth: u32) -> Vec<Option<T::Action>> {
        let (active, actions) = state.actions(self.player);
        if !active {
            return vec![None];
        }

        let mut actions = actions.into_iter();

        let mut best_actions = Vec::new();
        let mut best = if let Some(action) = actions.next() {
            let value = self.minimax(state, &action, completed_depth);
            best_actions.push(action);
            value
        } else {
            return vec![None];
        };

        for action in actions {
            let new = self.minimax(state, &action, completed_depth);
            match new.cmp(&best) {
                Ordering::Equal => best_actions.push(action),
                Ordering::Greater => {
                    best_actions.clear();
                    best_actions.push(action);
                    best = new;
                }
                Ordering::Less => (),
            }
        }

        let worst_allowed = best_actions
            .into_iter()
            .map(|action| self.minimax(state, &action, completed_depth + 1))
            .min()
            .unwrap();

        let mut actions: Vec<_> = state
            .actions(self.player)
            .1
            .into_iter()
            .filter(|action| self.minimax(state, action, completed_depth + 1) >= worst_allowed)
            .map(Some)
            .collect();
        actions
    }

    fn minimax(&mut self, state: &T, action: &T::Action, depth: u32) -> T::Fitness {
        if depth == 0 {
            state.look_ahead(&action, self.player)
        } else {
            let mut state = state.clone();
            let fitness = state.execute(&action, self.player);
            let (active, actions) = state.actions(self.player);

            let iter = actions
                .into_iter()
                .map(|action| self.minimax(&state, &action, depth - 1));

            if active { iter.max() } else { iter.min() }.unwrap_or(fitness)
        }
    }
}

impl<T: Game> Brute<T>
where
    T::Fitness: fmt::Debug,
    T::Action: fmt::Debug,
{
    pub fn print_best(&mut self, state: &T, depth: u32) {
        let (active, actions) = state.actions(self.player);
        assert!(active);

        let mut actions = actions.into_iter();

        let mut best = {
            let action = actions.next().unwrap();
            let value = self.minimax(state, &action, depth);
            (action, value)
        };

        for action in actions {
            let new = self.minimax(state, &action, depth);
            if new > best.1 {
                best = (action, new);
            }
        }

        println!("best: {:?}, fitness: {:?}", best.0, best.1);
    }
}
