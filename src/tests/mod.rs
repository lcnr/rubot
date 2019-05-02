use crate::{tree::Node, Bot, Depth, Logger, Steps, ToCompletion};

mod brute;
mod completed;
mod partial;

#[test]
fn logger_eq() {
    #[rustfmt::skip]
    let logger_eq = Node::root().push_children(&[
        Node::new(false, 0).push_children(&[
            Node::new(false, 0).push_children(&[
                Node::new(false, 0).push_children(&[
                    Node::new(false, 0).push_children(&[
                        Node::new(false, 0),
                    ]),
                ]),
            ]),
        ]),
        Node::new(false, 0).push_children(&[
            Node::new(true, 5).push_children(&[
                Node::new(true, 5).push_children(&[
                    Node::new(true, 3),
                ]),
            ]),
        ]),
        Node::new(false, 0).push_children(&[
            Node::new(false, 4),
            Node::new(false, 2).push_children(&[
                Node::new(false, 2),
            ]),
        ]),
    ]);

    let mut logger = Logger::new(Steps(7));
    Bot::new(true).select(&logger_eq, &mut logger);
    assert_eq!(logger.steps(), 7);

    let mut logger = Logger::new(Depth(2));
    Bot::new(true).select(&logger_eq, &mut logger);
    assert_eq!(logger.depth(), 2);
}
