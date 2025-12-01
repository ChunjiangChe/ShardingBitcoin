use crate::{
    types::{
        hash::{H256, Hashable},
        database::Database,
    },
    sharding_bitcoin::{
        block::{
            Info,
            versa_block::VersaBlock,
        },
        configuration::Configuration,  
    }
};
use std::{
    cmp,
    collections::{HashMap, VecDeque},
};
use std::time::{SystemTime};

#[derive(Clone)]
pub struct Node {
    pub val: H256,
    pub children: Vec<Box<Node>>,
    pub height: usize,
    pub longest_height: usize,
}

pub struct Blockchain {
    hash2blk: Database<VersaBlock>, //blk_hash -> block
    //Rust does not allow two pointers to point to the same variable
    hash2node: HashMap<H256, Node>, //blk_hash -> node
    root: Box<Node>,
    //multiple blocks
    pub longest_chain_hash: H256,
    pub height: usize,
    pub config: Configuration,
}

//prune the branches which are not growing on the longest chain. 
//should return the pruned blocks's hash to delete the corresponding states
impl Node {
    pub fn pre_traverse(root: &Box<Node>) -> Vec<H256> {
        let mut res: Vec<H256> = vec![root.val.clone()];
        for child in root.children.iter() {
            let t = Self::pre_traverse(child);
            res.extend(t);
        }
        res
    }
    pub fn insert(
        root: &mut Box<Node>, 
        parent: &H256, 
        hash: H256, 
        k: usize
    ) -> Option<Box<Node>>
    {
        if (&root.val).eq(parent) {
            //check whether the node exits
            //if exits, return that node and nothing would be deleted
            for n in root.children.iter() {
                if n.val == hash {
                    return Some(n.clone());
                }
            }
            //creating a new node. As there is only one new child, nothing would
            //be deleted
            let new_node: Box<Node> = Box::new(Node{
                val: hash,
                children: Vec::new(),
                height: root.height + 1,
                longest_height: root.height + 1
            });
            root.children.push(new_node.clone());
            if new_node.longest_height > root.longest_height {
                root.longest_height = new_node.longest_height;
            }
            Some(new_node)
        } else {
            let mut return_node: Option<Box<Node>> = None;
            for item in root.children.iter_mut() {
                let sub_return_node = Self::insert(item, parent, hash, k);
                match sub_return_node {
                    Some(res) => {
                        //If the new node is extending the longest chain, we gonna 
                        //delete something
                        if res.longest_height > root.longest_height {
                            root.longest_height = res.longest_height;
                        }
                        return_node = Some(res);
                        break;
                    }
                    None => {}
                }
                //Anyway, sub_pruned_nodes is Some only if sub_return_node is Some
                //but for beauty I split the logics of them
                
            }
            
            return_node 
        }
    }

    pub fn get_leaves(root: &Box<Node>) -> Vec<H256> {
        if root.children.is_empty() {
            let res: Vec<H256> = vec![root.val.clone()];
            res
        } else{
            let mut res: Vec<H256> = Vec::new();
            for child in root.children.iter() {
                let leaves = Self::get_leaves(child);
                res.extend(leaves);
            }
            res 
        }
    }
    //if pruning succeed, return all deleted hash, otherwise return None
    pub fn prune(root: &mut Box<Node>, hash: &H256) -> Option<Vec<H256>> {
         if root.children.is_empty() {
            None
        } else {
            let mut is_prune = false;
            let mut deleted_hash: Option<Vec<H256>> = None;
            for child in root.children.iter() {
                if (&child.val).eq(hash) {
                    is_prune = true;
                    deleted_hash = Some(Self::pre_traverse(child));
                    break;
                }
            }
            if is_prune {
                root.children.retain(|x| !(&x.val).eq(hash));
                root.longest_height = Self::get_longest_height(root);
            } else {
                for child in root.children.iter_mut() {
                    deleted_hash = Self::prune(child, hash);
                    if deleted_hash.is_some() {
                        root.longest_height = Self::get_longest_height(root);
                        break;
                    }
                }
            }
            deleted_hash
        }
    }

    fn get_longest_height(root: &Box<Node>) -> usize {
        if root.children.is_empty() {
            root.height
        } else {
            let mut longest_height = root.height;
            for child in root.children.iter() {
                longest_height = cmp::max(
                    longest_height, 
                    Self::get_longest_height(child)
                );
            }
            longest_height
        }
    }

    // fn get_longest_chain_hash(root: &Box<Node>) -> (H256, usize) {
    //     if root.children.is_empty() {
    //         (root.val.clone(), root.height)
    //     } else {
    //         let mut longest_height = root.height;
    //         let mut longest_hash = root.val.clone();
    //         for child in root.children.iter() {
    //             let (sub_hash, sub_height) = Self::get_longest_chain_hash(child);
    //             if sub_height > longest_height {
    //                 longest_height = sub_height;
    //                 longest_hash = sub_hash;
    //             }
    //         }
    //         (longest_hash, longest_height)
    //     }
    // }


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

    pub fn get_node_by_hash(root: &Box<Node>, hash: &H256) -> Option<Box<Node>> {
        if root.val == *hash {
            Some(root.clone())
        } else {
            for child in root.children.iter() {
                match Self::get_node_by_hash(child, hash) {
                    Some(node) => {
                        return Some(node);
                    }
                    None => {}
                }
            }
            None
        }
    }

    pub fn get_leaves_start_from(root: &Box<Node>, hash: &H256) -> Option<Vec<H256>> {
        if root.val == *hash {
            Some(Self::get_leaves(root))
        } else {
            for child in root.children.iter() {
                match Self::get_leaves_start_from(child, hash) {
                    Some(leaves) => {
                        return Some(leaves);
                    }
                    None => {}
                }
            }
            None
        }
    }

    
}

impl Blockchain {
    /// Create a new blockchain, only containing the genesis block
    pub fn new(genesis_block: VersaBlock, config: &Configuration) -> Self {
        let genesis_hash = genesis_block.hash();

        let now = SystemTime::now();
        //let mut hash2blk: HashMap<H256, VersaBlock> = HashMap::new();
        let mut hash2blk: Database<VersaBlock> = 
          Database::<VersaBlock>::new(format!("node(shard-{},index-{})/blockchain/hash2blk/{:?}", config.shard_id, config.node_id, now));
        hash2blk.insert(genesis_hash.clone(), genesis_block.clone()).unwrap();

        let root = Box::new(Node {
            val: genesis_hash.clone(),
            children: Vec::new(),
            height: 0,
            longest_height: 0,
        });
        let longest_chain_hash = genesis_hash.clone();
        let height = 0 as usize;
        // let verified_height = 0 as usize;
        let mut hash2node: HashMap<H256, Node> = HashMap::new();
        hash2node.insert(genesis_hash.clone(), (*root).clone());

        

    

        Blockchain {
            hash2blk,
            hash2node,
            root,
            longest_chain_hash,
            height,
            config: config.clone(),
        }
    }
    
    // fn delete_block(&mut self, hash: &H256) {
    //     self.hash2blk.remove(hash);
    //     self.hash2node.remove(hash);
    //     //self.tx_map.retain(|_, val| *hash != val.0);
    // }

    pub fn size(&self) -> usize {
        self.hash2node.len()
    }

    pub fn get_genesis_block(&self) -> VersaBlock {
        self.hash2blk.get(&(self.root.val)).unwrap()
    }


    pub fn insert_block_with_parent(&mut self, block: VersaBlock, parent: &H256) 
        -> Result<bool, String> 
    {
        let blk_hash = block.hash();
        if let Some(_) = self.hash2blk.get(&blk_hash) {
            return Err(String::from("Block already exits"));
        }
        
        //check whether the valid parent set contains the given parent
        if let None = self.hash2blk.get(&parent) {
            return Err(String::from("Parent doesn't exisit"));
        }
         
        let possible_node = Node::insert(
            &mut self.root,
            &parent,
            blk_hash.clone(),
            self.config.k
        );
        if let None = possible_node {
            return Err(String::from("Insertion fail"));
        }



        let new_node = possible_node.unwrap();
        //update hash2node
        self.hash2node.insert(blk_hash.clone(), (*new_node).clone());

        //update basic information
        self.hash2blk.insert(
            blk_hash.clone(),
            block.clone()
        ).unwrap();

        //update the longest chain information
        if new_node.height > self.height {
            self.height = new_node.height;
            self.longest_chain_hash = new_node.val.clone();
        } 

        
            
        Ok(true)
    }

    /// Get the last block's hash of the longest chain
    pub fn tip(&self) -> H256 {
        self.longest_chain_hash.clone()
    }

    /// Get all blocks' hashes of the longest chain, ordered from genesis to the tip
    pub fn all_blocks_in_longest_chain(&self) -> Vec<H256> {
        Node::get_path(&self.root, &self.longest_chain_hash)
                .unwrap()
    }

    

    //Get all blocks' hashs of the path end with specific hash
    pub fn all_blocks_end_with_block(&self, hash: &H256) -> Option<Vec<H256>> {
        Node::get_path(&self.root, hash)
    }

    // get the block from H256
    pub fn get_block(&self, hash: &H256) -> Option<VersaBlock> {
        match self.hash2blk.get(hash) {
            Some(block_ref) => {
                Some(block_ref.clone())
            }
            None => {
                None
            }
        }
    }


    

    pub fn is_block_confirmed(&self, hash: &H256, k: usize) -> bool {
        match Node::get_node_by_hash(&self.root, hash) {
            Some(node) => {
                node.longest_height - node.height >= k
            }
            None => {
                false
            }
        } 
    }


    pub fn get_block_height(&self, block_hash: &H256) -> Option<usize> {
        match self.hash2node.get(block_hash) {
            Some(node) => {
                Some(node.height)
            }
            None => None,
        }
    }



    pub fn get_forking_rate(&self) -> f64 {
        let main_chain_blocks = self.all_blocks_in_longest_chain();
        let main_chain_block_num = main_chain_blocks.len() as f64;
        let total_block_num = self.hash2blk.len() as f64;

        let forking_rate = main_chain_block_num / total_block_num;
        forking_rate
    }
    


}

