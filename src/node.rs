use anyhow::{anyhow, Result};
use core::fmt;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

const BITLENGTH: u8 = 8;

const MAX: u32 = 2u32.pow(BITLENGTH as u32) - 1;

type NodeRef = Rc<RefCell<NodeInner>>;
#[derive(Clone)]
pub struct Node {
    pub node_inner: NodeRef,
}

#[derive(Clone)]
struct Finger {
    start: u8,
    node: Option<Node>,
}
#[derive(Clone)]
pub struct FingerTable {
    node_id: u8,
    predecessor: Option<Node>,
    finger_table: Vec<Option<Finger>>,
}

#[derive(Clone)]
pub struct NodeInner {
    id: u8,
    finger_table: FingerTable,
    //key = key identifier/ finger id, aslo the index for fingertable, value = node identifier
    local_keys: HashMap<u8, Option<u8>>,
    lookup_info: Vec<String>,
}
impl Finger {
    fn new(start: u8, node: Option<Node>) -> Self {
        Self { start, node }
    }

    fn set_node(&mut self, node: Node) {
        self.node = Some(node);
    }
}
impl<'a> FingerTable {
    fn new(node_id: u8) -> Self {
        let mut finger_table = Vec::<Option<Finger>>::new();
        finger_table.push(Some(Finger::new(0, None)));
        for i in 1..=BITLENGTH {
            let finger_id = FingerTable::finger_id(node_id, i);
            finger_table.push(Some(Finger::new(finger_id, None)));
        }

        Self {
            node_id,
            predecessor: None,
            finger_table,
        }
    }

    fn set(&mut self, index: u8, successor: Node) {
        assert_ne!(index, 0);
        let offset = 2u32.pow((index - 1) as u32);
        let power = 2u32.pow(BITLENGTH as u32);
        let start = (self.node_id as u32 + offset) % power;
        self.finger_table[index as usize] = Some(Finger::new(start as u8, Some(successor)));
    }

    fn get(&self, index: u8) -> &Finger {
        assert_ne!(index, 0);
        self.finger_table[index as usize].as_ref().unwrap()
    }

    fn get_start(&self, index: u8) -> Result<u8> {
        assert_ne!(index, 0);
        if let Some(ref finger) = self.finger_table[index as usize] {
            Ok(finger.start)
        } else {
            Err(anyhow!(
                "Node {}: cannot find start of index {}",
                self.node_id,
                index
            ))
        }
    }

    fn get_successor_id(&self) -> u8 {
        if let Some(ref successor) = self.get(1).node {
            successor.node_inner.borrow().id
        } else {
            self.node_id
        }
    }

    fn get_successor_node(&self) -> Option<Node> {
        Some(Node::new_inner(Rc::clone(
            &self.get(1).node.as_ref().unwrap().node_inner,
        )))
    }

    fn set_successor(&mut self, node: Node) {
        //self.successor = Some(node);
        self.set(1, node);
    }

    fn set_predecessor(&mut self, predecessor: Option<Node>) {
        self.predecessor = predecessor;
    }

    fn get_predecessor_id(&self) -> u8 {
        if let Some(pre) = &self.predecessor {
            pre.node_inner.borrow().id
        } else {
            self.node_id
        }
    }
    pub fn finger_id(node_id: u8, index: u8) -> u8 {
        assert_ne!(index, 0);
        let offset: u128 = 2u128.pow((index - 1) as u32);
        let power: u128 = 2u128.pow(BITLENGTH as u32);
        let id = (node_id as u128 + offset) % power;
        //print!("finger id for index {} and node {}: {:#b}\n", index, node_id, id);
        id as u8
    }

    fn pretty_print(&self) {
        println!("----------Node id:{}----------", self.node_id);
        println!(
            "Successor:  {} Predecessor: {}",
            self.get_successor_id(),
            self.get_predecessor_id()
        );
        println!("FingerTables:");
        for (i, item) in self.finger_table.iter().enumerate() {
            if i == 0 {
                continue;
            }
            if let finger = item {
                let interval_right: u8;
                if i >= BITLENGTH as usize {
                    interval_right = self.node_id;
                } else {
                    interval_right = self.get_start((i + 1) as u8).unwrap();
                }
                println!(
                    "| k =  {} [ {} , {} )\tsucc. = {}\t|",
                    i,
                    finger.as_ref().unwrap().start,
                    interval_right,
                    finger
                        .as_ref()
                        .unwrap()
                        .node
                        .as_ref()
                        .unwrap()
                        .node_inner
                        .borrow()
                );
            }
        }
        println!("------------------------------");
    }
}

impl fmt::Display for NodeInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
impl NodeInner {
    pub fn new(node_id: u8) -> Self {
        Self {
            id: node_id,
            finger_table: FingerTable::new(node_id),
            local_keys: HashMap::<u8, Option<u8>>::new(),
            lookup_info: Vec::new(),
        }
    }
}

impl Node {
    pub fn new(node_id: u8) -> Self {
        Self {
            node_inner: Rc::new(RefCell::new(NodeInner::new(node_id))),
        }
    }
    pub fn new_inner(node_inner: NodeRef) -> Self {
        Self { node_inner }
    }

    pub fn join(&mut self, node: Option<Node>) -> Result<()> {
        if let Some(n) = node {
            self.init_finger_table(n.clone())?;
            self.update_others()?;
            self.transfer_keys();
            Ok(())
        // first node to join the chord
        } else {
            for i in 1..=BITLENGTH {
                self.node_inner
                    .borrow_mut()
                    .finger_table
                    .set(i, Self::new_inner(Rc::clone(&self.node_inner)));
            }
            self.node_inner
                .borrow_mut()
                .finger_table
                .set_predecessor(Some(Self::new_inner(Rc::clone(&self.node_inner))));
            Ok(())
        }
    }

    pub fn init_finger_table(&mut self, node: Node) -> Result<()> {
        let node_successor =
            node.find_successor(self.node_inner.borrow().finger_table.get_start(1)?)?;
        self.node_inner
            .borrow_mut()
            .finger_table
            .set_successor(node_successor);

        let predecessor = self.successor()?.predecessor();

        self.node_inner
            .borrow_mut()
            .finger_table
            .set_predecessor(predecessor);

        self.successor()?
            .node_inner
            .borrow_mut()
            .finger_table
            .set_predecessor(Some(Self::new_inner(Rc::clone(&self.node_inner))));

        for i in 1..=BITLENGTH - 1 {
            let self_id = self.node_inner.borrow().id;
            let node_inner = &mut self.node_inner.borrow_mut();
            let finger = node_inner.finger_table.get(i + 1).clone();
            let finger_pre = node_inner.finger_table.get(i).clone();
            // if (finger[i + 1].start belongs [n; finger[i].node))
            if finger_pre.node.is_some()
                && self.e_is_between_ring(
                    finger.start,
                    self_id,
                    finger_pre.node.as_ref().unwrap().node_inner.borrow().id,
                )
            {
                node_inner.finger_table.set(i + 1, finger_pre.node.unwrap());
                //finger.set_node(finger_pre.node.unwrap());
                //finger.node = finger_pre.node;
            } else {
                node_inner
                    .finger_table
                    .set(i + 1, node.find_successor(finger.start)?);
                //finger.set_node(node.find_successor(finger.start)?);
                //finger.node = Some(node.find_successor(finger.start)?);
            }
        }
        Ok(())
    }

    fn update_others(&mut self) -> Result<()> {
        for i in 1..=BITLENGTH {
            let offset = 2u8.pow((i - 1) as u32);
            let prev = Self::decrease(self.node_inner.borrow().id, offset);
            let mut p = self.find_predecessor(prev)?.clone();

            if prev == p.successor()?.node_inner.borrow().id {
                p = p.successor()?;
            }

            p.update_finger_table(Self::new_inner(Rc::clone(&self.node_inner)), i);
        }
        Ok(())
    }

    fn update_finger_table(&mut self, node: Node, index: u8) {
        assert_ne!(index, 0);
        let n_id = self.node_inner.borrow().id;
        let s_id = node.node_inner.borrow().id;

        if s_id != n_id {
            let f_id = self
                .node_inner
                .borrow()
                .finger_table
                .get(index)
                .node
                .as_ref()
                .unwrap()
                .node_inner
                .borrow()
                .id;
            if self.e_is_between_ring(s_id, n_id, f_id) {
                self.node_inner
                    .borrow_mut()
                    .finger_table
                    .set(index, node.clone());
                //self.node_inner.borrow().finger_table.get(index).node = Some(node.clone());
                let predecessor = self.predecessor();
                if let Some(mut pre) = predecessor {
                    pre.update_finger_table(node.clone(), index);
                }
            }
        }
    }

    fn update_finger_table_leave(&mut self, node: Node, index: u8, leav_id: u8) {
        let n_id = self.node_inner.borrow().id;
        let s_id = node.node_inner.borrow().id;

        if s_id != n_id {
            let f_id = self
                .node_inner
                .borrow()
                .finger_table
                .get(index)
                .node
                .as_ref()
                .unwrap()
                .node_inner
                .borrow()
                .id;
            if f_id == leav_id {
                self.node_inner
                    .borrow_mut()
                    .finger_table
                    .set(index, node.clone());
                //self.node_inner.borrow().finger_table.get(index).node = Some(node.clone());
                let predecessor = self.predecessor();
                if let Some(mut pre) = predecessor {
                    pre.update_finger_table_leave(node.clone(), index, leav_id);
                }
            }
        }
    }

    pub fn leave(&mut self) -> Result<()> {
        let successor = self.successor()?;
        let predecessor: Option<Node> = self.predecessor();

        successor
            .node_inner
            .borrow_mut()
            .finger_table
            .set_predecessor(predecessor);

        self.predecessor()
            .unwrap()
            .node_inner
            .borrow_mut()
            .finger_table
            .set_successor(successor);

        self.transfer_keys_leave();
        self.update_others_leave()?;
        Ok(())
    }

    fn update_others_leave(&self) -> Result<()> {
        let leave_id = self.node_inner.borrow().id;
        for i in 1..=BITLENGTH {
            let offset = 2u8.pow((i - 1) as u32);
            let prev = Self::decrease(self.node_inner.borrow().id, offset);
            let mut p = self.find_predecessor(prev)?.clone();
            p.update_finger_table_leave(self.successor()?, i, leave_id);
        }
        Ok(())
    }

    pub fn pretty_print(&self) {
        self.node_inner.borrow().finger_table.pretty_print();
    }

    pub fn print_keys(&self) {
        let id = self.node_inner.borrow().id;
        let key_len = self.node_inner.borrow().local_keys.len();
        println!("----------Node id:{}----------", id);
        print!("{{");
        for (i, (k, v)) in self.node_inner.borrow().local_keys.iter().enumerate() {
            let val: String;
            if v.is_none() {
                val = "None".to_string();
            } else {
                val = v.unwrap().to_string();
            }
            if i >= (key_len - 1) {
                print!("{}: {}", k, val);
            } else {
                print!("{}: {}, ", k, val);
            }
        }
        println!("}}");
    }

    pub fn print_lookup_results(&self) {
        let id = self.node_inner.borrow().id;
        println!("----------Node id:{}----------", id);
        for result in self.node_inner.borrow().lookup_info.iter() {
            println!("{}", result);
        }
        println!("------------------------------");
    }

    pub fn find(&self, key: u8) -> Option<u8> {
        let successor = self.find_successor(key).unwrap();
        let successor_id = successor.node_inner.borrow().id;
        let self_id = self.node_inner.borrow().id;
        if successor.node_inner.borrow().local_keys.contains_key(&key) {
            let value = successor.node_inner.borrow().local_keys[&key];
            let v: String;
            if value.is_none() {
                v = "None".to_string();
            } else {
                v = value.unwrap().to_string();
            }

            if successor_id == self_id {
                self.node_inner.borrow_mut().lookup_info.push(format!(
                    "Look-up result of key {} from node {} with path [{}] value is {}",
                    key, self_id, self_id, v
                ));
            } else {
                self.node_inner.borrow_mut().lookup_info.push(format!(
                    "Look-up result of key {} from node {} with path [{},{}] value is {}",
                    key, self_id, self_id, successor_id, v
                ));
            }
            value
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: u8, value: Option<u8>) {
        let successor = self.find_successor(key).unwrap();
        successor
            .node_inner
            .borrow_mut()
            .local_keys
            .insert(key, value);
    }

    fn remove(&mut self, key: u8) {
        let successor = self.find_successor(key).unwrap();
        successor.node_inner.borrow_mut().local_keys.remove(&key);
    }

    fn transfer_keys(&mut self) {
        let successor = self.successor().unwrap();
        let mut del_keys = Vec::<u8>::new();
        let mut migrations = Vec::<String>::new();
        let successor_id = successor.node_inner.borrow().id;
        let self_id = self.node_inner.borrow().id;
        for (k, v) in successor.node_inner.borrow().local_keys.iter() {
            let node = self.find_successor(*k).unwrap();
            // transfer key from successor to current node
            if node.node_inner.borrow().id == self.node_inner.borrow().id {
                migrations.push(format!(
                    "migrate key {} from node {} to node {}",
                    k, successor_id, self_id
                ));
                self.node_inner.borrow_mut().local_keys.insert(*k, *v);
                del_keys.push(*k);
            }
        }
        for key in del_keys {
            successor.node_inner.borrow_mut().local_keys.remove(&key);
        }

        if !migrations.is_empty() {
            println!("******************************");
            for migration in migrations {
                println!("{}", migration);
            }
        }
    }

    fn transfer_keys_leave(&mut self) {
        let mut successor = self.successor().unwrap();
        let mut del_keys = Vec::<u8>::new();
        let mut migrations = Vec::<String>::new();
        let successor_id = successor.node_inner.borrow().id;
        let self_id = self.node_inner.borrow().id;
        for (k, v) in self.node_inner.borrow().local_keys.iter() {
            // transfer key from current to successor node
            successor.insert(*k, *v);
            del_keys.push(*k);
            migrations.push(format!(
                "migrate key {} from node {} to node {}",
                k, self_id, successor_id
            ));
        }
        for key in del_keys {
            self.remove(key);
        }
        if !migrations.is_empty() {
            println!("******************************");
            for migration in migrations {
                println!("{}", migration);
            }
        }
    }

    fn successor(&self) -> Result<Node> {
        let binding = self.node_inner.borrow();
        let suc = binding.finger_table.get_successor_node();
        if let Some(s) = suc {
            Ok(Self::new_inner(Rc::clone(&s.node_inner)))
        } else {
            Err(anyhow!(
                "Node {}: successor is None",
                self.node_inner.borrow().id
            ))
        }
    }

    fn predecessor(&self) -> Option<Node> {
        self.node_inner.borrow().finger_table.predecessor.clone()
    }

    fn find_successor(&self, id: u8) -> Result<Node> {
        let n = self.find_predecessor(id)?;
        n.successor()
    }

    fn decrease(value: u8, size: u8) -> u8 {
        if size <= value {
            value - size
        } else {
            (MAX - (size - value) as u32) as u8
        }
    }
    fn find_predecessor(&self, id: u8) -> Result<Node> {
        let mut n = self.clone();
        while !self.is_between_ring_e(
            id,
            n.node_inner.borrow().id,
            n.successor()?.node_inner.borrow().id,
        ) {
            n = n.closest_preceding_node(id)?;
        }
        Ok(n)
    }

    fn is_between_ring_e(&self, id: u8, node1: u8, node2: u8) -> bool {
        if id == node2 {
            true
        } else {
            self.between(id, node1, node2)
        }
    }

    fn e_is_between_ring(&self, id: u8, node1: u8, node2: u8) -> bool {
        if id == node1 {
            true
        } else {
            self.between(id, node1, node2)
        }
    }

    fn between(&self, mut id: u8, mut node1: u8, mut node2: u8) -> bool {
        if node1 == node2 {
            return true;
        } else if node1 > node2 {
            let shift = MAX as u8 - node1;
            node1 = 0;
            node2 = (node2 + shift) % MAX as u8;
            id = ((id as u32 + shift as u32) % MAX) as u8;
        }
        node1 < id && id < node2 && node1 < node2
    }

    fn closest_preceding_node(&self, id: u8) -> Result<Node> {
        for i in (1..=BITLENGTH).rev() {
            let node_inner = self.node_inner.borrow();
            if node_inner.finger_table.get(i).node.is_some()
                && self.between(
                    node_inner
                        .finger_table
                        .get(i)
                        .node
                        .as_ref()
                        .unwrap()
                        .node_inner
                        .borrow()
                        .id,
                    node_inner.id,
                    id,
                )
            {
                return Ok(node_inner.finger_table.get(i).node.clone().unwrap());
            }
        }
        Ok(Self::new_inner(Rc::clone(&self.node_inner)))
    }
}
