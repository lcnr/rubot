//! runs `select` with a `RunCondition`.
//!
//! The returned action should either be the best action of the last completed depth
//! or have a fitness at the interrupted depth which is better than the fitness of the
//! best action from the previous depth.
#![no_main]

#[macro_use]
extern crate libfuzzer_sys;
extern crate rubot;

use rubot::{brute::Brute, Bot, Depth, Logger, Steps, ToCompletion, tree::Node};

fuzz_target!(|data: &[u8]| {
    if data.len() >= 4 {
        let node = Node::from_bytes(data);

        let (max_depth, max_steps) = {
            let mut logger = Logger::new(ToCompletion);
            Bot::new(true).select(&node, &mut logger);
            (logger.depth(), logger.steps())
        };

        for i in 0..max_steps {
            let mut logger = Logger::new(Steps(i));
            let selected = Bot::new(true).select(&node, &mut logger);
            if !Brute::new(true)
                .allowed_actions(&node, logger.depth())
                .into_iter()
                .find(|a| *a == selected)
                .is_some()
            {
                println!(
                    "Error with node: {:?}. Expected: {:?}, Actual: {:?}, Steps: {}",
                    node,
                    Brute::new(true).allowed_actions(&node, logger.depth()),
                    selected,
                    i
                );
                panic!();
            }
        }

        for i in 0..max_depth {
            let mut logger = Logger::new(Depth(i));
            let selected = Bot::new(true).select(&node, &mut logger);
            if !Brute::new(true).check_if_best(&node, selected.as_ref(), i) {
                println!(
                    "Error with node: {:?}. Expected: {:?}, Actual: {:?}, Depth: {}",
                    node,
                    Brute::new(true).allowed_actions(&node, logger.depth()),
                    selected,
                    i
                );
                panic!();
            }
        }
    }
});
