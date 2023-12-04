use core::fmt;
use std::collections::HashMap;
use anyhow::{Result, anyhow};

const BITLENGTH: usize = 8;

#[derive(Clone)]
struct Finger<'a>{
    start: u8,
    node: &'a Node,
}
#[derive(Clone)]
pub struct FingerTable<'a> {
    node_id: u8,
    predecessor: Option<Node>,
    finger_table: Vec<Option<Finger<'a>>>,
}

#[derive(Clone)]
pub struct Node{
    id: u8,
    //key = key identifier/ finger id, aslo the index for fingertable, value = node identifier
    local_keys: HashMap<u8, u8>,
}

#[derive(Clone)]
pub struct NodeStore<'a>{
    node: Node,
    finger_table : FingerTable<'a>,
}

impl<'a> Finger<'a>{
    fn new(start: u8, node: &'a Node) -> Self{
        Self{
            start, node,
        }
    }
    
    fn set_node(&mut self, node: &'a Node){
        self.node = node;
    }
}
impl<'a> FingerTable<'a>{
    fn new(node_id: u8) -> Self {
        let mut finger_table = Vec::<Option<Finger>>::new();
        //finger_table.reserve((2u32.pow(BITLENGTH as u32)+1) as usize);
        finger_table.resize(BITLENGTH+1, None);
        Self {
            node_id,
            predecessor: None,
            finger_table: finger_table,
        }
    }

    fn set(&mut self, index: u8, successor: &'a Node){
        let offset = 2u8.pow((index-1) as u32);
        let power = 2u8.pow(BITLENGTH as u32);
        let start = (self.node_id + offset) % power;
        self.finger_table.push(Some(Finger::new(start, successor)))
    }
    
    fn set_node(&mut self, index: u8, node: &'a Node) -> Result<()>{
        self.finger_table.get_mut(index as usize).ok_or(anyhow!("cannot get to index {} at finger table", index))?.as_mut().unwrap().node = node;
        Ok(())
    }

    fn get(&self, index: u8) -> Option<Finger>{
        self.finger_table[index as usize].clone()
    }
    
    fn get_start(&self, index: u8) -> Result<u8>{
        if let Some(ref finger) = self.finger_table[index as usize]{
            Ok(finger.start)
        } else {
            Err(anyhow!("cannot find start of index {}", index))
        }
    }

    fn get_successor_id(&self) -> u8{
        if let Some(successor) = self.get(0){
            successor.node.id
        }else{
            self.node_id
        }
    }

    fn set_predecessor(&mut self, predecessor: Option<Node>){
        self.predecessor = predecessor;
    }

    fn get_predecessor(&self) -> u8{
        if let Some(pre) = self.predecessor{
            pre.id
        }else{
            self.node_id
        }
    }
    pub fn finger_id(&self, node_id: u8, index: u8) -> u8{
        if index == 0{
            return node_id;
        }
        let offset: u128 = 2u128.pow((index-1) as u32);
        let power: u128 = 2u128.pow(BITLENGTH as u32);
        let id = (node_id as u128 + offset) % power;
        print!("finger id: {:#b}\n", id);
        id as u8
    }

    fn pretty_print(&self){
        println!("----------Node id:{}----------", self.node_id);
        println!("Successor: {} Predecessor: {}",self.get_successor_id(), self.get_predecessor());
        println!("Finger Tables:");
        for (i, item )in self.finger_table.iter().enumerate(){
            if let Some(n) = item{
                let interval_right: u8;
                if i >= BITLENGTH{
                    interval_right = self.node_id;
                }else{
                    interval_right = self.finger_table.get(i+1).as_ref().unwrap().as_ref().unwrap().start;
                }
                println!("| k = {} [{}, {})     succ. = {}", i, n.start, interval_right, n.node.successor().unwrap());
            }
        }
    }
}

impl fmt::Display for Node{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl Node{
    pub fn new(node_id: u8) -> Self{
        Self { id: node_id, local_keys: HashMap::new() }
    }
}
impl<'a> NodeStore <'a>{
    pub fn new(node_id: u8) -> Self {
        Self {
            node: Node::new(node_id),
            finger_table: FingerTable::new(node_id),
        }
    }

    pub fn init_finger_table(&'a mut self, node: &'a NodeStore) -> Result<()>{

        let node_successor  = node.find_successor(self.finger_table.get_start(1)?)?;
        self.finger_table.set(1, node_successor);
        let successor = self.successor()?;
        let predecessor = successor.predecessor()?;
        self.finger_table.set_predecessor(Some(predecessor));
        self.successor()?.finger_table.predecessor = Some(&self);
        for i in 1..BITLENGTH-1{
            let ref mut finger: &mut Finger = &mut self.finger_table.get((i+1) as u8).ok_or(anyhow!("cannot get to index {} at finger table", i+1))?;
            let finger_pre = self.finger_table.get(i as u8).ok_or(anyhow!("cannot get to index {} at finger table", i))?;
            if  finger.start >= self.id && finger.start < finger_pre.node.id{
                finger.node = finger_pre.node;
            }else{
                finger.node = node.find_successor(finger.start)?;
            }
        }
        Ok(())
    }

    fn join(&mut self, node: Option<Node>) -> Result<()>{
        /*
        if let Some(n) = node{
            self.init_finger_table(n);
            let successor = n.find_successor(self.id)?;
            // set successor
            self.finger_table.set(0, successor);
            return Ok(())
        // first node to join the chord
        } else{
            for i in 1..=BITLENGTH{
                self.finger_table.finger_table[i] = Some(self.clone());
                self.finger_table.set_predecessor(Some(&self));
            }
            return Ok(())
        }
        */
        unimplemented!()
    }
    
    fn stablize(&mut self){
        unimplemented!()
    }

    fn notify(&mut self, node: Node){
        unimplemented!()
    }

    fn find(&self, key: u8) -> u8 {
        unimplemented!()
    }

    fn insert(&mut self, key: u8) {
        unimplemented!()
    }

    fn remove(&mut self, key: u8) {
        unimplemented!()
    }

    fn successor(&self) -> Result<&NodeStore>{
        Ok(self.finger_table.get(0).as_ref().ok_or(anyhow!("unable to get successor of node: {}", self.id))?.node)
    }
    
    fn predecessor(&self) -> Result<&Node>{
        self.finger_table.predecessor.ok_or(anyhow!("unable to get predecessor"))
    }

    fn find_successor(&self, id: u8) -> Result<&Node>{
        if self.is_between_ring(id, self.id, self.successor()?.id){
            self.successor()
        }else{
            let n = self.closest_preceding_node(id)?;
            let successor = n.find_successor(id)?;
            Ok(successor)
        }
    }

    fn find_predecessor(&self, id: u8) -> Result<&Node>{
        if self.is_between_ring(id, self.id, self.successor()?.id){
            self.successor()
        } else{
            let n = self.closest_preceding_node(id)?;
            n.find_successor(id)
        }
    
    }

    fn is_between_ring(&self, id: u8, node1: u8, node2: u8) -> bool{
        if node1 < node2 {
            node1 < id && id <= node2
        } else {
            node1 < id || id <= node2
        }
    }

    fn closest_preceding_node(&self, id: u8) -> Result<&Node>{
        
        for finger in self.finger_table.finger_table.iter().rev(){
            match finger{
                Some(f) => {
                    if f.start > self.id && f.node.id < id && f.start < id{
                        return Ok(f.node);
                    }else if id < self.id{
                        return Ok(f.node);
                    }
                }
                None => {return Err(anyhow!("unable to find closest preceding node")); }
            }

        }
        Ok(self.successor()?)
    }

    fn fix_fingers(&'a mut self) -> Result<()>{
        for i in 0..self.finger_table.finger_table.len(){
            let finger_id = self.finger_table.finger_id(self.id, (i+1) as u8);
            if let Ok(successor) = self.find_successor(finger_id){
                self.finger_table.set_node(i as u8, successor)?;
            } 
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_build_node(){
        let node_id = 1;
        let mut node0 = Node::new(0);
        let mut node1 = Node::new(30);
        let mut node2 = Node::new(65);
        let mut node3 = Node::new(110);
        let mut node4 = Node::new(160);
        let mut node5 = Node::new(230);
        node0.join(None);
        node1.join(Some(node0));
        node2.join(Some(node1));
        node3.join(Some(node2));
        node4.join(Some(node3));
        node5.join(Some(node4));
    }
}