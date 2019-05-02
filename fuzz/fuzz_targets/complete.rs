#![no_main]
#[macro_use]
extern crate libfuzzer_sys;
extern crate rubot;

use rubot::{brute::Brute, tree::Node, Bot, ToCompletion};

fuzz_target!(|data: &[u8]| {
    if data.len() >= 4 {
        let node = Node::from_bytes(data);
        let selected = Bot::new(true).select(&node, ToCompletion);
        let is_best = Brute::new(true).check_if_best(&node, selected.as_ref(), std::u32::MAX);
        if !is_best {
            println!(
                "Error with node: {:?}. Expected: {:?}, Actual: {:?}",
                node,
                Brute::new(true).select(&node, std::u32::MAX),
                selected
            );
            panic!();
        }
    }
});
