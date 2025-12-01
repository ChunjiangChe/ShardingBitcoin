use crate::{
    types::{
        hash::{H256, Hashable},
    },
    sharding_bitcoin::{
        network::{
            message::Message,
            peer,
            server::Handle as ServerHandle,
        },
        block::{
            Info, 
            versa_block::{
                VersaBlock,
                VersaHash,
            }
        },
        configuration::Configuration,
        // validator::{Validator},
        mempool::Mempool,
        multichain::Multichain,
    }
};
use log::{debug, warn, error, info};
use std::{
    thread,
    sync::{Arc,Mutex},
    collections::{HashMap, VecDeque},
};

//#[cfg(any(test,test_utilities))]
//use super::peer::TestReceiver as PeerTestReceiver;
//#[cfg(any(test,test_utilities))]
//use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    multichain: Arc<Mutex<Multichain>>,
    mempool: Arc<Mutex<Mempool>>,
    config: Configuration,
    // validator: Validator,
    blk_buff: HashMap<VersaHash, Vec<VersaBlock>>,
    unavailable_cmt2avai_blocks: HashMap<H256, Vec<VersaBlock>>, //cmt -> avai blocks containing cmt
    unavailable_avai_block2cmts: HashMap<H256, Vec<H256>> // avai block hash -> cmts
}

// pub type SampleIndex = (H256, u32, u32); //block_hash, tx_index, shard_id
// pub type Sample = (u32, H256);

impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        multichain: &Arc<Mutex<Multichain>>,
        mempool: &Arc<Mutex<Mempool>>,
        config: &Configuration,
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            multichain: Arc::clone(multichain),
            blk_buff: HashMap::new(),
            mempool: Arc::clone(mempool),
            config: config.clone(),
            unavailable_cmt2avai_blocks: HashMap::new(),
            unavailable_avai_block2cmts: HashMap::new(),
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        info!("num of network workers: {num_worker}");
        for i in 0..num_worker {
            let mut cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }


    fn worker_loop(&mut self) {
        loop {
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }

                
                Message::NewBlockHash(hash_vec) => {
                    //debug!("New versa block hash");
                    if let Some(response) = self
                        .handle_new_block_hash(hash_vec) {
                        peer.write(response);
                    }
                }
                Message::GetBlocks(hash_vec) => {
                    //debug!("Get versa blocks");
                    if let Some(response) = self
                        .handle_get_blocks(hash_vec) {
                        peer.write(response);
                    }
                }
                Message::Blocks(blocks) => {
                    //debug!("Coming versa blocks");
                    let (response_1, response_2) = self
                        .handle_blocks(blocks); 
                    if let Some(new_blks) = response_1 {
                        self.server.broadcast(new_blks);
                    }

                    //handle missing blocks
                    if let Some(missing_blks) = response_2 {
                        peer.write(missing_blks);
                    }

                }
                
            }
        }
    }
   
    
    fn handle_new_block_hash(
        &self, 
        block_hash_vec: Vec<VersaHash>) -> Option<Message> 
    {
        if block_hash_vec.is_empty() {
            return None;
        }

        let mut unreceived_blks: Vec<VersaHash> = vec![];

        for versa_hash in block_hash_vec {
            match versa_hash.clone() {
                VersaHash::OrderHash(order_hash) => {
                    match self.multichain
                        .lock()
                        .unwrap()
                        .get_order_block(
                        &order_hash) {
                        Some(_) => {}
                        None => unreceived_blks.push(
                            versa_hash
                        ),
                    }
                }
                VersaHash::ShardHash(shard_hash) => {
                    let mut is_found = false;
                    //not sure the shard id of the exclusive block based on its hash
                    for id in 0..self.config.shard_num {
                        match self.multichain
                            .lock()
                            .unwrap()
                            .get_shard_block_by_shard(
                            &shard_hash,
                            id
                        ){
                            Some(_) => {
                                is_found = true;
                                break;
                            }
                            None => {}
                        }
                    }
                    if !is_found {
                        unreceived_blks.push(
                            versa_hash
                        );
                    }
                }
            }
        }
            
        

        if !unreceived_blks.is_empty() {
            Some(Message::GetBlocks(unreceived_blks))
        } else {
            None
        }
    }

    fn handle_get_blocks(&self, hash_vec: Vec<VersaHash>) 
        -> Option<Message>
    {
        if hash_vec.is_empty() {
            return None;
        }
        

        let mut res_blks: Vec<VersaBlock> = vec![];

        for versa_hash in hash_vec {
            match versa_hash {
                VersaHash::OrderHash(order_hash) => {
                    match self.multichain
                        .lock()
                        .unwrap()
                        .get_order_block(
                            &order_hash
                    ){
                        Some(block) => res_blks.push(VersaBlock::OrderBlock(block)),
                        None => {}
                    }
                }
                VersaHash::ShardHash(shard_hash) => {
                    for id in 0..self.config.shard_num {
                        match self.multichain
                            .lock()
                            .unwrap()
                            .get_shard_block_by_shard(
                            &shard_hash, 
                            id
                        ){
                            Some(block) => {
                                res_blks.push(VersaBlock::ShardBlock(block));
                                break;
                            }
                            None => {}
                        }
                    }
                }
            }
        }
        
        

        if !res_blks.is_empty() {
            Some(Message::Blocks(res_blks))
        } else {
            None
        }
    }

    fn handle_blocks(&mut self, blocks: Vec<VersaBlock>) 
        -> (Option<Message>, Option<Message>) 
    //new_block_hash, missing block, missing symbols
    {
        if blocks.is_empty() {
            return (None, None);
        }

        
        let mut new_hashs: Vec<VersaHash> = vec![];
        let mut missing_parents: Vec<VersaHash> = vec![];
        
        // return tx
        for block in blocks {
            //verification
            //verify if hash is valid
            if !block.verify_hash() {
                // return Err(String::from("Incorrect hash"));
                info!("Reject block {:?} for incorrect hash", block.hash());
                continue;
            }
            let block_hash = block.hash();
            info!("Incoming block {:?}", block_hash);
            
            // let shard_id = block.get_shard_id();
            //insert the block
            let (sub_new_hashes, sub_missing_parents) = self.insert_block(block.clone());
            new_hashs.extend(sub_new_hashes);
            missing_parents.extend(sub_missing_parents);
        }


        let res_new_hashes = match new_hashs.is_empty() {
            true => None,
            false => Some(Message::NewBlockHash(new_hashs)),
        };

        let res_missing_blks = match missing_parents.is_empty() {
            true => None,
            false => Some(Message::GetBlocks(missing_parents)),
        };
        

        (res_new_hashes, res_missing_blks)
    }

    fn insert_block(&mut self, block: VersaBlock) -> (Vec<VersaHash>, Vec<VersaHash>) {
        let mut new_hashs: Vec<VersaHash> = vec![];
        // let mut missing_parents: HashMap<usize, Vec<H256>> = HashMap::new();
        let mut missing_parents: Vec<VersaHash> = vec![];
        let parents: Vec<(VersaHash, usize)> = match block.clone() {
            VersaBlock::OrderBlock(order_block) => {
                vec![(VersaHash::OrderHash(order_block.get_order_parent()), 0)]
            }
            VersaBlock::ShardBlock(shard_block) => {
                vec![(VersaHash::ShardHash(shard_block.get_shard_parent()), block.get_shard_id())]
            }
        };
        
        for item in parents {
            let parent_hash = item.0;
            let inserted_shard_id = item.1;
            // //this is important
            // //the inclusive block can not be inserted in his own shard
            // if let VersaBlock::InBlock(_) = block {
            //     if inserted_shard_id == self.config.shard_id &&
            //         block.get_shard_id() == self.config.shard_id {
            //         continue;
            //     }
            // }
            
            //check whether the parent exits
            let mut parent_not_exisit = false;
            match parent_hash.clone() {
                VersaHash::OrderHash(order_hash) => {
                    match self.multichain
                        .lock()
                        .unwrap()
                        .get_order_block(&order_hash) {
                        Some(_) => {}
                        None => {
                            parent_not_exisit = true;
                        }
                    }
                }
                VersaHash::ShardHash(shard_hash) => {
                    match self.multichain
                        .lock()
                        .unwrap()
                        .get_shard_block_by_shard(&shard_hash, inserted_shard_id) {
                        Some(_) => {}
                        None => {
                            parent_not_exisit = true;
                        }
                    }
                }
            }

            //put the block in buff
            if parent_not_exisit {
                match self.blk_buff.get(&parent_hash) {
                    Some(old_blks) => {
                        if !old_blks.contains(&block) {
                            let mut new_blks = old_blks.clone();
                            new_blks.push(block.clone());
                            self.blk_buff.insert(parent_hash.clone(), new_blks);
                        }
                    }
                    None => {
                        self.blk_buff.insert(parent_hash.clone(), vec![block.clone()]);
                    }
                }
                
                info!("block {:?} insertion failure in shard {}: parent {:?} not fould", block.hash(), inserted_shard_id, parent_hash);
                if !missing_parents.contains(&parent_hash) {
                    missing_parents.push(parent_hash.clone());
                }
                continue;
            }                

            // match self.validator.validate_block(&block) {
            //     Ok(_) => {}
            //     Err(e) => {
            //         info!("block insertion failure: the verification fails: {:?}", e);
            //         continue;
            //     }
            // }
            

            let mut inserted_blks: VecDeque<VersaBlock> = VecDeque::new();
            inserted_blks.push_back(block.clone());
            let mut removed_buff: Vec<VersaHash> = vec![];
            while !inserted_blks.is_empty() {
                let inserted_blk = inserted_blks.pop_front().unwrap();
                match self.multichain
                    .lock()
                    .unwrap()
                    .insert_block_with_parent(
                    inserted_blk.clone(),
                    &parent_hash
                ) {
                    Ok(_) => {
                        let new_hash = match inserted_blk.clone() {
                            VersaBlock::OrderBlock(_) 
                                => VersaHash::OrderHash(inserted_blk.hash()),
                            VersaBlock::ShardBlock(_)
                                => VersaHash::ShardHash(inserted_blk.hash()),
                        };
                        new_hashs.push(new_hash.clone());
                        info!("successfully inserting block: {:?}", new_hash);
                        

                        //if there are some blocks in the buff whose parent is the new block,
                        //continue to insert it
                        match self.blk_buff.get(&new_hash) {
                            Some(child_blks) => {
                                for child_blk in child_blks {
                                    inserted_blks.push_back(child_blk.clone());
                                }
                                removed_buff.push(new_hash);
                            }
                            None => {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        info!("Reject block {:?} in shard {}: insertion fails: {}", inserted_blk.hash(), self.config.shard_id, e);
                        break;
                    }
                }
            }
            for item2 in removed_buff {
                self.blk_buff.remove(&item2);
            }
        }
        (new_hashs, missing_parents)
    }
}

