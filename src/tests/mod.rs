use crate::{tree::Node, Bot, Depth, Logger, Steps, ToCompletion};

mod brute;
mod completed;
mod partial;

#[rustfmt::skip]
const LOGGER_EQ: Node = Node::root().children(&[
    Node::new(false, 0).children(&[
        Node::new(false, 0).children(&[
            Node::new(false, 0).children(&[
                Node::new(false, 0).children(&[
                    Node::new(false, 0),
                ]),
            ]),
        ]),
    ]),
    Node::new(false, 0).children(&[
        Node::new(true, 5).children(&[
            Node::new(true, 5).children(&[
                Node::new(true, 3),
            ]),
        ]),
    ]),
    Node::new(false, 0).children(&[
        Node::new(false, 4),
        Node::new(false, 2).children(&[
            Node::new(false, 2),
        ]),
    ]),
]);

#[test]
fn logger_eq() {
    let mut logger = Logger::new(Steps(7));
    Bot::new(true).select(&LOGGER_EQ, &mut logger);
    assert_eq!(logger.steps(), 7);

    let mut logger = Logger::new(Depth(2));
    Bot::new(true).select(&LOGGER_EQ, &mut logger);
    assert_eq!(logger.depth(), 2);
}
