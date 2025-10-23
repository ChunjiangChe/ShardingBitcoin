use crate::{
    types::{
        hash::{H256, Hashable},
    },
    bitcoin::{
        block::Block,
        transaction::{
            Transaction,
            TxFlag,
        }
    }
};
use std::collections::{HashMap, VecDeque};
use log::{debug};

#[derive(Clone)]
pub struct Node {
    val: H256,
    children: Vec<Box<Node>>,
    height: usize,
}


pub type State = HashMap<(String, u32), Transaction>;

pub struct Blockchain {
    hash_map: HashMap<String, Block>,
    root: Box<Node>,
    tx_map: HashMap<String, (String, usize)>, //tx_hash -> (block_hash, index)
    pub states: HashMap<String, State>, //block_hash -> state
    pub longest_chain_hash: H256,
    pub height: usize,
}

//If available, prune the branches which are not growing on the longest chain. 
//However it is not neccessary
impl Node {
    pub fn insert(root: &mut Box<Node>, parent: &H256, hash: &H256) -> Option<Box<Node>> {
        if (&root.val).eq(parent) {
            let new_node: Box<Node> = Box::new(Node{
                val: hash.clone(),
                children: Vec::new(),
                height: root.height + 1,
            });
            root.children.push(new_node.clone());
            Some(new_node)
        } else {
            let mut res: Option<Box<Node>> = None;
            for item in root.children.iter_mut() {
                res = Self::insert(item, parent, hash);
                if res.is_some() {
                    break;
                }
            }
            res
        }
    }

    pub fn get_path(root: &Box<Node>, hash: &H256) -> Option<Vec<H256>> {
        if (&root.val).eq(hash) {
            let mut res: Vec<H256> = Vec::new();
            res.push(hash.clone());
            Some(res)
        } else {
            let mut res: Vec<H256> = Vec::new();
            for item in root.children.iter() {
                match Self::get_path(item, hash) {
                    Some(ret) => {
                        res.push(root.val.clone());
                        res.extend(ret);
                        break;
                    }
                    None => {}
                }
            }
            if res.is_empty() {
                None
            } else {
                Some(res)
            }
        }
    }

    pub fn print_tree(root: &Box<Node>) {
        let mut queue: VecDeque<&Box<Node>> = VecDeque::new();
        queue.push_back(root);
        while !queue.is_empty() {
            let mut tvec: Vec<&Box<Node>> = Vec::new();
            while let Some(node) = queue.pop_back() {
                tvec.push(node);
            }
            for item in tvec.iter() {
                print!("{} ", hex::encode(&item.val.0));
                for item2 in item.children.iter() {
                    queue.push_back(item2);
                }
            }
            println!("");
        }
    }
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new(difficulty: &H256) -> Self {
        //create genesis block
        let mut genesis_block: Block = Block::new();
        genesis_block.set_difficulty(difficulty);
        let mut hash_map: HashMap<String, Block> = HashMap::new();
        //convert the type of `hash` from `H56` to `String` for being stored in the HashMap
        let hash_str: String = hex::encode(&genesis_block.hash().0);
        hash_map.insert(hash_str.clone(), genesis_block.clone());
        let root: Box<Node> = Box::new(Node {
            val: genesis_block.hash(),
            children: Vec::new(),
            height: 1,
        });
        let longest_chain_hash: H256 = genesis_block.hash();
        let height: usize = 1;

        //intitialize a empty state to the genesis block
        let state: State = HashMap::new();
        let mut states: HashMap<String, State> = HashMap::new();
        states.insert(hash_str, state);

        Blockchain {
            hash_map,
            root,
            tx_map: HashMap::new(),
            states,
            longest_chain_hash,
            height,
        }
    }


    /// Insert a block into blockchain
    pub fn insert(&mut self, block: &Block) -> (bool, bool) {
        let blk_hash: H256 = block.hash();
        let hash_str: String = hex::encode(&blk_hash.0);
        if let Some(_) = self.hash_map.get(&hash_str) {
            (false, false)
        } else {
            let possible_node: Option<Box<Node>> = Node::insert(&mut self.root, &block.get_parent(), &blk_hash);
            match possible_node {
                Some(new_node) => {
                    let mut extend_or_not = false;
                    self.hash_map.insert(hash_str.clone(), block.clone());
                    if new_node.height > self.height {
                        self.height = new_node.height;
                        self.longest_chain_hash = new_node.val.clone();
                        extend_or_not = true;
                    } else {
                        debug!("Fork occurs");
                    }
                    let txs = &block.content.txs.data;
                    let parent_hash = block.get_parent();
                    let mut state: State = self.states.get(&hex::encode(&parent_hash.0)).unwrap().clone();
                    (0..txs.len()).for_each(|i| {
                        let tx = &txs[i];
                        let tx_hash = tx.hash();
                        let tx_str = hex::encode(&tx_hash);
                        self.tx_map.insert(tx_str.clone(), (hash_str.clone(), i));
                        //update the state
                        if let TxFlag::Initial = tx.flag {
                            state.insert((tx_str.clone(), 0), tx.clone());
                        } else {
                            for input in tx.inputs.iter() {
                                state.remove(&(hex::encode(&input.tx_hash), input.index)).unwrap();
                            }
                            for j in 0..tx.outputs.len() {
                                state.insert((tx_str.clone(), j as u32), tx.clone());
                            }
                        }  
                    });
                    self.states.insert(hash_str.clone(), state);
                    (true, extend_or_not)
                }
                None => {
                    debug!("Parent of block not found");
                    (false, false)
                }
            }
        }
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        self.longest_chain_hash.clone()
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        let longes_chain_blocks: Vec<H256> = Node::get_path(&self.root, &self.longest_chain_hash).unwrap();
        longes_chain_blocks
        
    }

    // get the block from H256
    pub fn get_block(&self, hash: &H256) -> Option<Block> {
        let hash_str: String = hex::encode(hash.0);
        match self.hash_map.get(&hash_str) {
            Some(block_ref) => {
                Some(block_ref.clone())
            }
            None => {
                None
            }
        }
    }

    pub fn get_tx_in_longest_chain(&self, tx_hash: &H256) -> Option<Transaction> {
        if let Some((blk_hash_str, index)) = self.tx_map.get(&hex::encode(&tx_hash)) {
            let blk = self.hash_map.get(blk_hash_str).unwrap();
            let blk_hash = blk.hash();
            let longest_chain_blks: Vec<H256> = self.all_blocks_in_longest_chain();
            if longest_chain_blks.contains(&blk_hash) {
                let tx: Transaction = blk.content.txs.data[*index].clone();
                Some(tx)
            } else {
                None
            }
        } else {
            None
        }
    }


}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::types::block::generate_random_block;
//    use crate::types::hash::Hashable;
//
//    #[test]
//    fn insert_one() {
//        let difficulty: H256 = (&[255u8; 32]).into();
//        let mut blockchain = Blockchain::new(&difficulty);
//        let genesis_hash = blockchain.tip();
//        let block = generate_random_block(&genesis_hash);
//        blockchain.insert(&block);
//        assert_eq!(blockchain.tip(), block.hash());
//
//    }
//
//    #[test] 
//    fn insert_multiple() {
//        let difficulty: H256 = (&[255u8; 32]).into(); 
//        let mut blockchain: Blockchain = Blockchain::new(&difficulty);
//        let genesis_hash: H256 = blockchain.tip();
//        let fan_num: usize = 3;
//        let depth: usize = 3;
//        let mut queue: VecDeque<H256> = VecDeque::new();
//        queue.push_back(genesis_hash);
//        (0..depth).for_each(|_| {
//            let mut tvec: Vec<H256> = Vec::new();
//            while let Some(thash) = queue.pop_front() {
//                tvec.push(thash);
//            }
//            for item in tvec.iter() {
//                (0..fan_num).for_each(|_| {
//                    let block: Block = generate_random_block(item);
//                    blockchain.insert(&block);
//                    queue.push_back(block.hash());
//                });
//            }
//        });
//        Node::print_tree(&blockchain.root);
//        assert_eq!(blockchain.height, 4); 
//    }
//
//}
//
//// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
