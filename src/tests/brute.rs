//! testing tests, deep!
use super::*;
use crate::brute::Brute;

#[test]
fn allowed_actions_depth_zero() {
    #[rustfmt::skip]
    let allowed_actions_depth_zero = Node::root().push_children(&[
        Node::new(true, -1),
        Node::new(true, 65).push_children(&[
            Node::new(false, 0)
        ]),
        Node::new(true, 11),
    ]);

    const EXPECTED: &[Option<usize>] = &[Some(1), Some(2)];

    let mut actual = Brute::new(true).allowed_actions(&allowed_actions_depth_zero, 0);
    assert_eq!(EXPECTED.len(), actual.len(), "actual: {:?}", actual);

    for item in EXPECTED.iter() {
        if let Some(pos) = actual.iter().position(|act| act == item) {
            actual.remove(pos);
        } else {
            panic!("actual: {:?}", actual);
        }
    }
}
