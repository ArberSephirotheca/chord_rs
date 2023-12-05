#[cfg(test)]
mod tests {

    use super::super::node::Node;
    #[test]
    fn test_sample_case() {
        let mut n0 = Node::new(0);
        let mut n1 = Node::new(30);
        let mut n2 = Node::new(65);
        let mut n3 = Node::new(110);
        let mut n4 = Node::new(160);
        let mut n5 = Node::new(230);
        n0.join(None);
        n1.join(Some(n0.clone()));
        n2.join(Some(n1.clone()));
        n3.join(Some(n2.clone()));
        n4.join(Some(n3.clone()));
        n5.join(Some(n4.clone()));
        n0.pretty_print();
        n1.pretty_print();
        n2.pretty_print();
        n3.pretty_print();
        n4.pretty_print();
        n5.pretty_print();

        n0.insert(3, Some(3));
        n1.insert(200, None);
        n2.insert(123, None);
        n3.insert(45, Some(3));
        n4.insert(99, None);
        n2.insert(60, Some(10));
        n0.insert(50, Some(8));
        n3.insert(100, Some(5));
        n3.insert(101, Some(4));
        n3.insert(102, Some(6));
        n5.insert(240, Some(8));
        n5.insert(250, Some(10));

        n0.print_keys();
        n1.print_keys();
        n2.print_keys();
        n3.print_keys();
        n4.print_keys();
        n5.print_keys();

        let mut n6 = Node::new(100);
        n6.join(Some(n5.clone()));
        n3.print_keys();
        n6.print_keys();

        n0.find(3);
        n0.find(200);
        n0.find(123);
        n0.find(45);
        n0.find(99);
        n0.find(60);
        n0.find(50);
        n0.find(100);
        n0.find(101);
        n0.find(102);
        n0.find(240);
        n0.find(250);

        n2.find(3);
        n2.find(200);
        n2.find(123);
        n2.find(45);
        n2.find(99);
        n2.find(60);
        n2.find(50);
        n2.find(100);
        n2.find(101);
        n2.find(102);
        n2.find(240);
        n2.find(250);

        n6.find(3);
        n6.find(200);
        n6.find(123);
        n6.find(45);
        n6.find(99);
        n6.find(60);
        n6.find(50);
        n6.find(100);
        n6.find(101);
        n6.find(102);
        n6.find(240);
        n6.find(250);

        n0.print_lookup_results();
        n2.print_lookup_results();
        n6.print_lookup_results();

        n2.leave();

        n0.pretty_print();
        n1.pretty_print();

        n0.print_keys();
        n1.print_keys();
    }
}
