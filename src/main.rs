mod client;
mod node;
mod test;
use node::Node;

//mod node_store;
fn main() {
    let mut node0 = Node::new(0);
    let mut node1 = Node::new(30);
    let mut node2 = Node::new(65);
    let mut node3 = Node::new(110);
    let mut node4 = Node::new(160);
    let mut node5 = Node::new(230);
    node0.join(None);
    //node1.join(Some(Node::new_inner(Rc::clone(&node0.node_inner))));
    node1.join(Some(node0.clone()));
    node2.join(Some(node1.clone()));
    //node2.join(Some(Node::new_inner(Rc::clone(&node0.node_inner))));
    node3.join(Some(node2.clone()));
    node4.join(Some(node3.clone()));
    node5.join(Some(node4.clone()));
    node0.pretty_print();
    node1.pretty_print();
    node2.pretty_print();
}
