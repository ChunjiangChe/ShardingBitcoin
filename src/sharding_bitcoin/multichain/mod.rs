use crate::{
    sharding_bitcoin::{
        blockchain::Blockchain,
        configuration::Configuration,
        block::{
            Info,
            versa_block::{VersaHash, VersaBlock},
            OrderBlock,
            ShardBlock,
        },
    },
    types::{
        hash::{H256, Hashable},
        database::Database,
    }
};
// use std::{
//     // sync::{Arc, Mutex},
//     collections::BTreeSet,
// };
use std::time::{SystemTime};


pub struct Multichain {
    pub config: Configuration,
    order_chain: Blockchain,
    shard_chains: Vec<Blockchain>,
    confirmed_shard_blocks: Vec<Vec<H256>>,
    longest_order_chain: Vec<H256>,
}

// impl Clone for Multichain {
//     fn clone(&self) -> Self {
//         let new_availability_chains: Vec<Blockchain> = self.availability_chains
//             .iter()
//             .map(|x| x.clone())
//             .collect();
//         Multichain {
//             config: self.config.clone(),
//             proposer_chain: self.proposer_chain.clone(),
//             availability_chains: new_availability_chains,
//             new_tx_blocks: BTreeSet::new(),
//         }
//     }
// }

impl Multichain {
    pub fn new(
        order_chain: Blockchain,
        shard_chains: Vec<Blockchain>, 
        config: &Configuration) -> Self 
    {   

        Multichain {
            order_chain,
            shard_chains,
            confirmed_shard_blocks: vec![],
            longest_order_chain: vec![],
            config: config.clone(),
        }
    }

    pub fn insert_block_with_parent(
        &mut self,
        block: VersaBlock,
        parent: &VersaHash
    ) -> Result<bool, String> {
        // let blk_hash = block.hash();
        match parent.clone() {
            VersaHash::OrderHash(h) => {
                match self.order_chain
                    .insert_block_with_parent(block.clone(), &h) {
                    Ok(_) => {
                        self.longest_order_chain = self.order_chain.all_blocks_in_longest_chain();
                        Ok(true)
                    }
                    Err(e) => Err(e),
                }
            }
            VersaHash::ShardHash(h) => {
                let shard_id = block.get_shard_id();
                let insert_success = match self.shard_chains
                    .get_mut(shard_id)        
                    .unwrap()
                    .insert_block_with_parent(block.clone(), &h) {
                    Ok(_) => Ok(true),
                    Err(e) => Err(e),
                };
                match insert_success {
                    Ok(_) => {
                        let longest_shard_chain = self.shard_chains
                            .get(shard_id)
                            .unwrap()
                            .all_blocks_in_longest_chain();
                        let n = longest_shard_chain.len().saturating_sub(self.config.k);
                        let confirmed_shard_blocks = longest_shard_chain[..n].to_vec();
                        if let Some(ele) = self.confirmed_shard_blocks
                            .get_mut(shard_id) {
                                *ele = confirmed_shard_blocks;
                        } else {
                            panic!("Overflow");
                        }

                    }
                    Err(_) => {}
                }
                insert_success
            }
        }
    }


    pub fn all_blocks_in_longest_order_chain(&self) -> Vec<H256> {
        self.order_chain
            .all_blocks_in_longest_chain()

    }
    pub fn all_blocks_in_longest_shard_chain_by_shard(&self, shard_id: usize) -> Vec<H256> {
        self.shard_chains
            .get(shard_id)
            .unwrap()
            .all_blocks_in_longest_chain()
    }
    pub fn all_order_blocks_end_with_block(&self, hash: &H256) -> Option<Vec<H256>> {
        self.order_chain
            .all_blocks_end_with_block(hash)
    }
    // pub fn get_tx_blk_in_longest_proposer_chain(
    //     &self, 
    //     blk_hash: &H256) -> Option<TransactionBlock> 
    // {
    //     self.proposer_chain
    //         .get_tx_blk_in_longest_chain(blk_hash)
    // }
    pub fn get_highest_order_block(&self) -> H256 {
        self.order_chain
            .tip()
    }
    pub fn get_highest_shard_block(&self, shard_id: usize) -> H256 {
        self.shard_chains
            .get(shard_id)
            .unwrap()
            .tip()
    }

    pub fn get_all_highest_shard_blocks(&self) -> Vec<(H256, usize)> {
        (0..self.config.shard_num)
            .into_iter()
            .map(|i| (self.shard_chains.get(i).unwrap().tip(), i))
            .collect()
    }

    pub fn get_new_confirmed_shard_blocks(&self) -> Vec<H256> {
        let all_confirmed_shard_blocks = self.confirmed_shard_blocks.concat();
        
        let new_confirmed_shard_blocks: Vec<H256> = all_confirmed_shard_blocks
            .into_iter()
            .filter(|x| !self.longest_order_chain.contains(x))
            .collect();
        new_confirmed_shard_blocks
    }

    
    pub fn get_order_block(&self, hash: &H256) -> Option<OrderBlock> {
        match self.order_chain.get_block(hash) {
            Some(versa_block) => {
                if let VersaBlock::OrderBlock(order_block) = versa_block {
                    Some(order_block)
                } else {
                    panic!("Non-order block exists in order chain");
                }
            }
            None => None,
        }
    }
    pub fn get_shard_block_by_shard(&self, hash: &H256, shard_id: usize) -> Option<ShardBlock> {
        match self.shard_chains  
            .get(shard_id)
            .unwrap()
            .get_block(hash) {
            Some(versa_block) => {
                match versa_block {
                    VersaBlock::OrderBlock(_) => panic!("Non-order block exists in order chains"),
                    VersaBlock::ShardBlock(shard_block) => Some(shard_block),
                }
            }
            None => None,
        } 
    }

    pub fn get_order_size(&self) -> usize {
        self.order_chain.size()
    }

    pub fn get_shard_size(&self, shard_id: usize) -> usize {
        self.shard_chains.get(shard_id).unwrap().size()
    }
    
    pub fn print_order_chain(&self) {
        let all_order_hashes = self.order_chain.all_blocks_in_longest_chain();
        for order_hash in all_order_hashes.iter() {
            let block = self.order_chain.get_block(order_hash).unwrap();
            if let VersaBlock::OrderBlock(order_block) = block {
                println!("{:?}\n", order_block);
            } else {
                panic!("Not an order block");
            }
        }
        println!("");
    }

    pub fn print_shard_chains(&self) {
        for i in 0..self.config.shard_num {
            let all_shard_hashes = self
                .shard_chains
                .get(i)
                .unwrap()
                .all_blocks_in_longest_chain();
            for shard_hash in all_shard_hashes.iter() {
                let block = self.shard_chains
                    .get(i)
                    .unwrap()
                    .get_block(shard_hash).unwrap();
                match block {
                    VersaBlock::OrderBlock(_) => {
                        panic!("Not a shard block");
                    }
                    VersaBlock::ShardBlock(shard_block) => {
                        println!("Shard {} {:?}\n", i, shard_block);
                    }
                }
            }
            println!("");
        }
    }

    pub fn get_order_forking_rate(&self) -> f64 {
        self.order_chain.get_forking_rate()
    }

    pub fn get_shard_forking_rate_by_shard(&self, shard_id: usize) -> f64 {
        self.shard_chains
            .get(shard_id)
            .unwrap()
            .get_forking_rate()
    }

    // pub fn get_all_prop_refer_tx_blks(&self) -> Vec<TransactionBlock> {
    //     //to be completed
    //     vec![]
    // }
    // pub fn log_to_file_with_shard(&self, shard_id: usize) {
    //     self.chains
    //         .get(shard_id)
    //         .unwrap()
    //         .lock()
    //         .unwrap()
    //         .log_to_file();
    // }
}
