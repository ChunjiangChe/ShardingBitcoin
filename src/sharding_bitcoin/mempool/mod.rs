use crate::{
    types::{
        hash::{
            H256, Hashable,
        },
        database::Database,
    },
    sharding_bitcoin::{
        configuration::Configuration,
        transaction::Transaction,
    },
};
use std::collections::{VecDeque};
// use log::{info, debug};
use std::time::{SystemTime};


pub struct Mempool {
    tx_map: Database<Transaction>, //the key is the hash of the tx block, while value is the
    //full block
    tx_queue: VecDeque<H256>,
}


impl Mempool {
    pub fn new(config: &Configuration) -> Self {
        let now = SystemTime::now();
        let tx_map: Database<Transaction> = 
            Database::<Transaction>::new(format!("node(shard-{},index-{})/mempool/tx_map/{:?}", config.shard_id, config.node_id, now));
        Mempool {
            tx_map,
            tx_queue: VecDeque::new(),
        }
    }

    pub fn get_size(&self) -> usize {
        self.tx_map.len()
    }

    pub fn get_queue_size(&self) -> usize {
        self.tx_queue.len()
    }
    
    pub fn get_txs(&mut self, num: usize) -> Result<Vec<Transaction>, Vec<Transaction>> {
        let mut txs: Vec<Transaction> = vec![];
        if num >= self.tx_queue.len() {
            while !self.tx_queue.is_empty() {
                let hash = self.tx_queue.pop_front().unwrap();
                let tx = self.tx_map.get(&hash).unwrap().clone();
                self.tx_map.remove(&hash);
                txs.push(tx);
            }
            Ok(txs)
        } else {
            for _i in 0..num {
                let hash = self.tx_queue.pop_front().unwrap();
                let tx = self.tx_map.get(&hash).unwrap().clone();
                self.tx_map.remove(&hash);
                txs.push(tx);
            }
            Err(txs)
        }
        
    }

    pub fn insert_tx(&mut self, tx: Transaction) -> bool {
        let hash = tx.hash();
        if self.tx_map.contains_key(&hash) {
            //block already exists.
            false
        } else {
            self.tx_map.insert(hash.clone(), tx.clone()).unwrap();
            self.tx_queue.push_back(hash);
            true
        }
    }

    pub fn check(&self, hash: &H256) -> bool {
        self.tx_map.contains_key(hash)
    }
        
    pub fn get_tx(&self, hash: &H256) -> Option<Transaction> {
        if self.check(hash) {
            Some(self.tx_map.get(hash).unwrap().clone())
        } else {
            None
        }
    }

    pub fn get_all_txs(&self) -> Vec<Transaction> {

        self.tx_map
            .iter()
            .map(|(_, val)| val.clone())
            .collect()

    }

    pub fn delete_txs(&mut self, tx_hashs: Vec<H256>) -> bool {
        for hash in tx_hashs.iter() {
            self.tx_map.remove(&hash);
            self.tx_queue.retain(|x| x != hash);
        }
        true
    }

    

    pub fn pop_one_tx(&mut self) -> Option<Transaction> {
        if self.tx_queue.is_empty() {
            None
        } else {
            let hash = self.tx_queue.pop_front().unwrap();
            let tx_blk = self.tx_map.get(&hash).unwrap().clone();
            self.tx_map.remove(&hash);
            Some(tx_blk)
        }
    }

    pub fn get_all_tx_hash(&self) -> Vec<H256> {
        self.tx_map
            .iter()
            .map(|(key, _)| key.clone())
            .collect()
    }
    
}

