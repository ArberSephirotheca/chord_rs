mod node;
use std::rc::Rc;

use node::Node;
//mod node_store;
fn main() {
    println!("Hello, world!");
    let mut node0 = Node::new(0);
    let mut node1 = Node::new(30);
    let mut node2 = Node::new(65);
    let mut node3 = Node::new(110);
    let mut node4 = Node::new(160);
    let mut node5 = Node::new(230);
    let result = node0.join(None);
    println!("{:#?}", result);
    let result = node1.join(Some(Node::new_inner(Rc::clone(&node0.node_inner))));
    println!("{:#?}", result);
    node2.join(Some(node1.clone()));
    node3.join(Some(node2.clone()));
    node4.join(Some(node3.clone()));
    node5.join(Some(node4.clone()));
    node3.pretty_print();
}
