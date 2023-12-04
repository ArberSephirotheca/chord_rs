use core::fmt;
use std::{collections::HashMap, cell::RefCell, rc::Rc};
use anyhow::{Result, anyhow};

const BITLENGTH: usize = 8;

type NodeRef = Rc<RefCell<NodeInner>>;
#[derive(Clone)]
pub struct Node{
    node_inner: NodeRef
}

#[derive(Clone)]
struct Finger{
    start: u8,
    node:  Node,
}
#[derive(Clone)]
pub struct FingerTable {
    node_id: u8,
    predecessor: Option<Node>,
    finger_table: Vec<Option<Finger>>,
}

#[derive(Clone)]
pub struct NodeInner{
    id: u8,
    finger_table: FingerTable,
    //key = key identifier/ finger id, aslo the index for fingertable, value = node identifier
    local_keys: HashMap<u8, u8>,
}
impl Finger{
    fn new(start: u8, node: Node) -> Self{
        Self{
            start, node,
        }
    }
    
    fn set_node(&mut self, node: Node){
        self.node = node;
    }
}
impl<'a> FingerTable{
    fn new(node_id: u8) -> Self {
        let mut finger_table = Vec::<Option<Finger>>::new();
        for i in 1..=(BITLENGTH+1){
            let finger_id = FingerTable::finger_id(node_id, i as u8);
            finger_table.push(None);
        }
        finger_table.reserve((2u32.pow(BITLENGTH as u32)+1) as usize);
        
        Self {
            node_id,
            predecessor: None,
            finger_table: finger_table,
        }
    }

    fn set(&mut self, index: u8, successor: Node){
        let offset = 2u8.pow((index-1) as u32);
        let power = 2u8.pow(BITLENGTH as u32);
        let start = (self.node_id + offset) % power;
        self.finger_table[index as usize] = Some(Finger::new(start, successor));
    }
    
    fn set_node(&mut self, index: u8, node: Node) -> Result<()>{
        self.finger_table.get_mut(index as usize).ok_or(anyhow!("cannot get to index {} at finger table", index))?.as_mut().unwrap().node = node;
        Ok(())
    }

    fn get(&self, index: u8) -> Finger{
        self.finger_table[index as usize].as_ref().unwrap().clone()
    }
    
    fn get_start(&self, index: u8) -> Result<u8>{
        if let Some(ref finger) = self.finger_table[index as usize]{
            Ok(finger.start)
        } else {
            Err(anyhow!("cannot find start of index {}", index))
        }
    }

    fn get_successor_id(&self) -> u8{
        if let successor = self.get(0){
            successor.node.node_inner.borrow().id
        }else{
            self.node_id
        }
    }
    
    fn set_succesor(&mut self, node: Node){
        self.get(0).node = node;
    }

    fn set_predecessor(&mut self, predecessor: Option<Node>){
        self.predecessor = predecessor;
    }

    fn get_predecessor(&self) -> u8{
        if let Some(pre) = &self.predecessor{
            pre.node_inner.borrow().id
        }else{
            self.node_id
        }
    }
    pub fn finger_id(node_id: u8, index: u8) -> u8{
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
        println!("----------NodeInner id:{}----------", self.node_id);
        println!("Successor: {} Predecessor: {}",self.get_successor_id(), self.get_predecessor());
        println!("Finger Tables:");
        for (i, item )in self.finger_table.iter().enumerate(){
            if let n = item{
                let interval_right: u8;
                if i >= BITLENGTH{
                    interval_right = self.node_id;
                }else{
                    interval_right = self.finger_table.get(i+1).as_ref().unwrap().as_ref().unwrap().start;
                }
                println!("| k = {} [{}, {})     succ. = {}", i, n.as_ref().unwrap().start, interval_right, n.as_ref().unwrap().node.successor().unwrap().node_inner.borrow());
            }
        }
    }
}

impl fmt::Display for NodeInner{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}
impl NodeInner{
    pub fn new(node_id: u8) -> Self {
        Self {
            id : node_id,
            finger_table: FingerTable::new(node_id),
            local_keys: HashMap::<u8, u8>::new(),
        }
    }
}

impl Node{
    pub fn new(node_id: u8) -> Self{
        Self{
            node_inner: Rc::new(RefCell::new(NodeInner::new(node_id)))
        }
    }
    pub fn new_inner(node_inner: NodeRef) -> Self{
        Self{
            node_inner
        }
    }
    
    pub fn init_finger_table(&mut self, node: Node) -> Result<()>{

        let node_successor  = node.find_successor(self.node_inner.borrow().finger_table.get_start(1)?)?;
        self.node_inner.borrow_mut().finger_table.set(1, node_successor);
        let successor = self.successor()?;
        let predecessor = successor.predecessor();
        self.node_inner.borrow_mut().finger_table.set_predecessor(predecessor);
        self.successor()?.node_inner.borrow_mut().finger_table.predecessor = Some(Self::new_inner(Rc::clone(&self.node_inner)));
        for i in 1..BITLENGTH-1{
            let ref mut finger: &mut Finger = &mut self.node_inner.borrow_mut().finger_table.get((i+1) as u8);
            let finger_pre = self.node_inner.borrow().finger_table.get(i as u8);
            if  finger.start >= self.node_inner.borrow().id && finger.start < finger_pre.node.node_inner.borrow().id{
                finger.node = finger_pre.node;
            }else{
                finger.node = node.find_successor(finger.start)?;
            }
        }
        Ok(())
    }
    fn update_finger(&mut self, i: usize) -> Result<()> {
        let mut finger = self.node_inner.borrow_mut().finger_table.get((i + 1) as u8);
        let finger_pre = self.node_inner.borrow().finger_table.get(i as u8);
    
        if finger.start >= self.node_inner.borrow().id && finger.start < finger_pre.node.node_inner.borrow().id {
            finger.node = finger_pre.node;
        } else {
            finger.node = self.successor()?.find_successor(finger.start)?;
        }
    
        Ok(())
    }
    fn join(&mut self, node: Option<Node>) -> Result<()>{
        if let Some(n) = node{
            self.init_finger_table(n.clone())?;
            let successor = n.find_successor(self.node_inner.borrow().id)?;
            // set successor
            self.node_inner.borrow_mut().finger_table.set(0, successor);
            return Ok(())
        // first node to join the chord
        } else{
            for i in 1..=BITLENGTH{
                let finger_id = FingerTable::finger_id(self.node_inner.borrow().id, i as u8);
                self.node_inner.borrow_mut().finger_table.finger_table[i] = Some(Finger::new(finger_id, Self::new_inner(Rc::clone(&self.node_inner))));
                self.node_inner.borrow_mut().finger_table.set_predecessor(Some(Self::new_inner(Rc::clone(&self.node_inner))));
            }
            return Ok(())
        }
    }
    /*
    fn join_stablize(&mut self, node: Option<Node>) -> Result<()>{
        
        self.finger_table.set_predecessor(None);
        if let Some(n) = node{
            self.finger_table.set_succesor(n);
            Ok(())
        }else{
            Ok(())
        }
    }
    */
    
    fn update_others(&mut self) -> Result<()>{
        for i in 1..=BITLENGTH{
            let offset = 2u8.pow((i-1) as u32);
            let id = self.node_inner.borrow().id - offset;
            let mut p = self.find_predecessor(id)?.clone();
            p.update_finger_table(Self::new_inner(Rc::clone(&self.node_inner)), i as u8);
        }
        Ok(())
    }
    
    
    fn update_finger_table(&mut self, node: Node, index: u8){
        
    }
    /*
    fn stablize(&mut self) -> Result<()>{
        let n = self.successor()?;
        let result = n.borrow().predecessor();
        if let Some(x) = result{
            if self.is_between_ring(x.borrow().id, self.id, n.borrow().id){
            self.finger_table.set_succesor(Rc::clone(&x));
            }
        }

        let mut new_successor = self.successor()?;
        new_successor.borrow_mut().notify(Rc::new(RefCell::new(*self)))
    }

    fn notify(&mut self, node: Node) -> Result<()>{
        let predecessor = self.predecessor();
        if predecessor.is_none() || self.is_between_ring(node.borrow().id, predecessor.unwrap().borrow().id, self.id){
            self.finger_table.set_predecessor(Some(node));
        }
        Ok(())
    }
    */

    fn find(&self, key: u8) -> u8 {
        unimplemented!()
    }

    fn insert(&mut self, key: u8) {
        unimplemented!()
    }

    fn remove(&mut self, key: u8) {
        unimplemented!()
    }
    
    fn successor(&self) -> Result<Node>{
        Ok(self.node_inner.borrow().finger_table.get(0).node)
    }
    
    fn predecessor(&self) -> Option<Node>{
        self.node_inner.borrow().finger_table.predecessor.clone()
    }

    fn find_successor(&self, id: u8) -> Result<Node>{
        if self.is_between_ring(id, self.node_inner.borrow().id, self.successor()?.node_inner.borrow().id){
            self.successor()
        }else{
            let n = self.closest_preceding_node(id)?;
            let successor = n.find_successor(id)?;
            Ok(successor)
        }
    }

    fn find_predecessor(&mut self, id: u8) -> Result<Node>{
        if self.is_between_ring(id, self.node_inner.borrow().id, self.successor()?.node_inner.borrow().id){
            Ok(Node::new_inner(Rc::clone(&self.node_inner)))
            
        }else{
            let mut n = self.closest_preceding_node(id)?;
            let predecessor = n.find_predecessor(id)?;
            Ok(predecessor)
        }
    }

    
    fn is_between_ring(&self, id: u8, node1: u8, node2: u8) -> bool{
        if node1 < node2 {
            node1 < id && id <= node2
        } else {
            node1 < id || id <= node2
        }
    }
    
    fn closest_preceding_node(&self, id: u8) -> Result<Node>{
        
        for finger in self.node_inner.borrow().finger_table.finger_table.iter().rev(){
            match finger{
                Some(f) => {
                    if f.start > self.node_inner.borrow().id && f.node.node_inner.borrow().id < id && f.start < id{
                        return Ok(f.node.clone());
                    }else if id < self.node_inner.borrow().id{
                        return Ok(f.node.clone());
                    }
                }
                None => {return Err(anyhow!("cannot find closest peceding node"));}
            }

        }
        Ok(self.successor()?)
    }
    /*
    fn fix_fingers(&mut self) -> Result<()>{
        for i in 0..self.finger_table.finger_table.len(){
            let finger_id = FingerTable::finger_id(self.id, (i+1) as u8);  
            if let Ok(successor) = self.find_successor(finger_id){
                self.finger_table.finger_table[i].as_mut().unwrap().node = successor.clone();
            } 
        }
        Ok(())
    }
    */
    
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