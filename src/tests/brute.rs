//! testing tests, deep!
use super::*;
use crate::brute::Bot as Brute;

#[rustfmt::skip]
const ALLOWED_ACTIONS_DEPTH_ZERO: Node = Node::root().children(&[
    Node::new(true, -1),
    Node::new(true, 65).children(&[
        Node::new(false, 0)
    ]),
    Node::new(true, 11),
]);

#[test]
fn allowed_actions_depth_zero() {
    const EXPECTED: &[Option<usize>] = &[Some(1), Some(2)];

    let mut actual = Brute::new(true).allowed_actions(&ALLOWED_ACTIONS_DEPTH_ZERO, 0);
    assert_eq!(EXPECTED.len(), actual.len(), "actual: {:?}", actual);

    for item in EXPECTED.iter() {
        if let Some(pos) = actual.iter().position(|act| act == item) {
            actual.remove(pos);
        } else {
            panic!("actual: {:?}", actual);
        }
    }
}
