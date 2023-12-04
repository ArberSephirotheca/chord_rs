use core::fmt;
use std::{collections::HashMap, cell::RefCell, rc::Rc};
use anyhow::{Result, anyhow};

const BITLENGTH: usize = 8;

#[derive(Clone)]
struct NodeRef(Rc<RefCell<Node>>);

#[derive(Clone)]
struct Finger{
    start: u8,
    node:  NodeRef,
}
#[derive(Clone)]
pub struct FingerTable {
    node_id: u8,
    predecessor: Option<NodeRef>,
    finger_table: Vec<Option<Finger>>,
}

#[derive(Clone)]
pub struct Node{
    id: u8,
    finger_table: FingerTable,
    //key = key identifier/ finger id, aslo the index for fingertable, value = node identifier
    local_keys: HashMap<u8, u8>,
}
impl Finger{
    fn new(start: u8, node: NodeRef) -> Self{
        Self{
            start, node,
        }
    }
    
    fn set_node(&mut self, node: NodeRef){
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

    fn set(&mut self, index: u8, successor: NodeRef){
        let offset = 2u8.pow((index-1) as u32);
        let power = 2u8.pow(BITLENGTH as u32);
        let start = (self.node_id + offset) % power;
        self.finger_table[index as usize] = Some(Finger::new(start, successor));
    }
    
    fn set_node(&mut self, index: u8, node: NodeRef) -> Result<()>{
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
            successor.node.borrow().id
        }else{
            self.node_id
        }
    }
    
    fn set_succesor(&mut self, node: NodeRef){
        self.get(0).node = node;
    }

    fn set_predecessor(&mut self, predecessor: Option<NodeRef>){
        self.predecessor = predecessor;
    }

    fn get_predecessor(&self) -> u8{
        if let Some(pre) = &self.predecessor{
            pre.borrow().id
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
        println!("----------Node id:{}----------", self.node_id);
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
                println!("| k = {} [{}, {})     succ. = {}", i, n.as_ref().unwrap().start, interval_right, n.as_ref().unwrap().node.borrow().successor().unwrap().borrow());
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
    pub fn new(node_id: u8) -> Self {
        Self {
            id : node_id,
            finger_table: FingerTable::new(node_id),
            local_keys: HashMap::<u8, u8>::new(),
        }
    }
    
    pub fn init_finger_table(self_ : Rc<RefCell<Self>>, node: NodeRef) -> Result<()>{

        let node_successor  = node.borrow().find_successor(self_.borrow().finger_table.get_start(1)?)?;
        self_.borrow_mut().finger_table.set(1, node_successor);
        let successor = self_.borrow().successor()?;
        let predecessor = successor.borrow().predecessor();
        self_.borrow_mut().finger_table.set_predecessor(predecessor);
        self_.borrow_mut().successor()?.borrow_mut().finger_table.predecessor = Some(Rc::clone(&self_));
        for i in 1..BITLENGTH-1{
            let ref mut finger: &mut Finger = &mut self_.borrow().finger_table.get((i+1) as u8);
            let finger_pre = self_.borrow().finger_table.get(i as u8);
            if  finger.start >= self_.borrow().id && finger.start < finger_pre.node.borrow().id{
                finger.node = finger_pre.node;
            }else{
                finger.node = node.borrow().find_successor(finger.start)?;
            }
        }
        Ok(())
    }
    fn update_finger(&mut self, i: usize) -> Result<()> {
        let ref mut finger: &mut Finger = &mut self.finger_table.get((i + 1) as u8);
        let finger_pre = self.finger_table.get(i as u8);
    
        if finger.start >= self.id && finger.start < finger_pre.node.borrow().id {
            finger.node = finger_pre.node;
        } else {
            finger.node = self.successor()?.borrow().find_successor(finger.start)?;
        }
    
        Ok(())
    }
    fn join(&mut self, node: Option<NodeRef>) -> Result<()>{
        if let Some(n) = node{
            self.init_finger_table(n)?;
            let successor = n.borrow().find_successor(self.id)?;
            // set successor
            self.finger_table.set(0, successor);
            return Ok(())
        // first node to join the chord
        } else{
            for i in 1..=BITLENGTH{
                let finger_id = FingerTable::finger_id(self.id, i as u8);
                self.finger_table.finger_table[i] = Some(Finger::new(finger_id, Rc::new(RefCell::new(*self))));
                self.finger_table.set_predecessor(Some(Rc::new(RefCell::new(*self))));
            }
            return Ok(())
        }
    }

    fn join_stablize(&mut self, node: Option<NodeRef>) -> Result<()>{
        
        self.finger_table.set_predecessor(None);
        if let Some(n) = node{
            self.finger_table.set_succesor(n);
            Ok(())
        }else{
            Ok(())
        }
    }
    
    fn update_others(&mut self) -> Result<()>{
        for i in 1..=BITLENGTH{
            let offset = 2u8.pow((i-1) as u32);
            let mut p = self.find_predecessor(self.id - offset)?;
            p.borrow_mut().update_finger_table(Rc::new(RefCell::new(*self)), i as u8);
        }
        Ok(())
    }
    
    
    fn update_finger_table(&mut self, node: NodeRef, index: u8){
        
    }

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

    fn notify(&mut self, node: NodeRef) -> Result<()>{
        let predecessor = self.predecessor();
        if predecessor.is_none() || self.is_between_ring(node.borrow().id, predecessor.unwrap().borrow().id, self.id){
            self.finger_table.set_predecessor(Some(node));
        }
        Ok(())
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
    
    fn successor(&self) -> Result<NodeRef>{
        Ok(self.finger_table.get(0).node.clone())
    }
    
    fn predecessor(&self) -> Option<NodeRef>{
        self.finger_table.predecessor
    }

    fn find_successor(&self, id: u8) -> Result<NodeRef>{
        if self.is_between_ring(id, self.id, self.successor()?.borrow().id){
            self.successor()
        }else{
            let n = self.closest_preceding_node(id)?;
            let successor = n.borrow().find_successor(id)?;
            Ok(successor)
        }
    }

    fn find_predecessor(&mut self, id: u8) -> Result<NodeRef>{
        if self.is_between_ring(id, self.id, self.successor()?.borrow().id){
            Ok(Rc::new(RefCell::new(*self)))
            
        }else{
            let mut n = self.closest_preceding_node(id)?;
            let predecessor = n.borrow().find_predecessor(id)?;
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
    
    fn closest_preceding_node(&self, id: u8) -> Result<NodeRef>{
        
        for finger in self.finger_table.finger_table.iter().rev(){
            match finger{
                f => {
                    if f.as_ref().unwrap().start > self.id && f.as_ref().unwrap().node.borrow().id < id && f.as_ref().unwrap().start < id{
                        return Ok(f.as_ref().unwrap().node);
                    }else if id < self.id{
                        return Ok(f.as_ref().unwrap().node);
                    }
                }
            }

        }
        Ok(self.successor()?)
    }

    fn fix_fingers(&mut self) -> Result<()>{
        for i in 0..self.finger_table.finger_table.len(){
            let finger_id = FingerTable::finger_id(self.id, (i+1) as u8);  
            if let Ok(successor) = self.find_successor(finger_id){
                self.finger_table.finger_table[i].as_mut().unwrap().node = successor.clone();
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
        node1.join(Some(Rc::new(RefCell::new(node0))));
        node2.join(Some(Rc::new(RefCell::new(node1))));
        node3.join(Some(Rc::new(RefCell::new(node2))));
        node4.join(Some(Rc::new(RefCell::new(node3))));
        node5.join(Some(Rc::new(RefCell::new(node4))));
    }
}