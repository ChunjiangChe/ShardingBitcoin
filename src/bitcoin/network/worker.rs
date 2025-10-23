use crate::{
    types::{
        hash::{H256, Hashable},
    },
    bitcoin::{
        network::{
            message::Message,
            peer,
            server::Handle as ServerHandle,
        },
        blockchain::{
            Blockchain,
            State,
        },
        transaction::{Mempool, Transaction, UtxoInput, TxFlag},
        block::Block,
        configuration::Configuration,
    }
};
use log::{debug, warn, error};
use std::{
    thread,
    sync::{Arc,Mutex},
    collections::HashMap,
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
    blockchain: Arc<Mutex<Blockchain>>,
    blk_buff: HashMap<String, Block>,
    mempool: Arc<Mutex<Mempool>>,
    config: Configuration,
}

#[derive(Clone)]
pub enum ValidationSource {
    FromBlock,
    FromTransaction,
}

impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
        mempool: &Arc<Mutex<Mempool>>,
        config: &Configuration
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: Arc::clone(blockchain),
            blk_buff: HashMap::new(),
            mempool: Arc::clone(mempool),
            config: config.clone()
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let mut cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn check_input_from_state(input: &UtxoInput, state: &State) -> bool {
        match state.get(&(hex::encode(&input.tx_hash), input.index)) {
            Some(tx) => {
                let sig_vec = input.sig_ref.clone();
                let index: usize = input.index as usize;
                let output = tx.outputs.get(index).unwrap();
                let pub_key = output.public_key_ref.clone();
                match Transaction::verify(tx, pub_key.as_slice(), sig_vec.as_slice()) {
                    true => true,
                    false => {
                        debug!("signatrue is not valid");
                        false
                    }
                }
            }
            None => {
                false
            }
        }
    }

    fn validate_input(&self, input: &UtxoInput, history: &Vec<H256>, flag: ValidationSource) -> bool {
        let blockchain_states = self.blockchain
            .lock()
            .unwrap()
            .states.clone();

        let last_confirmed_blk_hash = history
            .get(history.len() - 1 - self.config.k)
            .unwrap();
        let confirmed_state = blockchain_states
            .get(&hex::encode(last_confirmed_blk_hash))
            .unwrap();
        
        if !Self::check_input_from_state(input, &confirmed_state) {
            debug!("input not found");
            return false;
        }
        
        let last_blk_hash = history.get(history.len() - 1).unwrap();
        let latest_state = blockchain_states
            .get(&hex::encode(last_blk_hash))
            .unwrap();
        if !Self::check_input_from_state(input, &latest_state) {
            debug!("input already used in the following blocks");
            return false;
        }
        
        if let ValidationSource::FromTransaction = flag {
            //the input must not exit in the mempool
            let mempool_txs = self.mempool.lock().unwrap().get_all_tx_ref();
            for tx in mempool_txs {
                for sub_input in tx.inputs.iter() {
                    if sub_input.tx_hash == input.tx_hash && sub_input.index == input.index {
                        debug!("Double spent");
                        return false;
                    }
                }
            }
        }
        //if the input utxo of the tx is not found, check the balance
        true
    }

    fn validate_initial_tx(&self, tx: &Transaction, history: &Vec<H256>) -> bool {
        let output = tx.outputs.get(0).unwrap();
        if output.value != self.config.initial_balance {
            debug!("Not valid intial transaction");
            return false;
        }
        let history_blks: Vec<Block> =
            (0..history.len()).map(|i| 
                self.blockchain.lock().unwrap().get_block(&history[i]).unwrap()    
            ).collect();
        //check whether there is another initial transaction for the same address exits
        for i in 0..(history.len() - self.config.k) {
            let txs = &history_blks[i].content.txs.data;
            for ttx in txs.iter() {
                //check whether is an initial transaction
                if let TxFlag::Initial = ttx.flag {
                    let ottx = ttx.outputs.get(0).unwrap();
                    if ottx.receiver_addr == output.receiver_addr {
                        debug!("double initial transactions");
                        return false;
                    }
                }
            }
        }
        true
    }

    pub fn validate_tx(&self, tx: &Transaction, flag: ValidationSource) -> bool {
        //1.check the format of the tx (e.g. the total output value
        //should not exceed the total input value) 
        //2.check the validity of the tx (double spent, signatrue)
        let longest_chain_hashs: Vec<H256> = 
            self.blockchain.lock().unwrap().all_blocks_in_longest_chain();

        //check whether the tx is creating the initial balance
        if let TxFlag::Initial = tx.flag {
            return self.validate_initial_tx(tx, &longest_chain_hashs); 
        }

        let mut available_coins: u32 = 0;
        let mut spent_coins: u32 = 0;
        for input in tx.inputs.iter() {
            available_coins += input.value;
            if !self.validate_input(input, &longest_chain_hashs, flag.clone()) {
                return false;
            }
        }

        for output in tx.outputs.iter() {
            spent_coins += output.value;
        }

        if available_coins != spent_coins {
            debug!("transaction format is not valid: the input value and output value must be equal");
            return false;
        }
        true
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
                Message::NewBlockHashes(hashs) => {
                    debug!("New block hashs");
                    let mut unreceived_blks: Vec<H256> = Vec::new();
                    for hash in hashs.iter() {
                        match self.blockchain.lock().unwrap().get_block(hash) {
                            Some(_) => {}
                            None => {
                                unreceived_blks.push(hash.clone());
                            }
                        }
                        
                    }
                    if !unreceived_blks.is_empty() {
                        peer.write(Message::GetBlocks(unreceived_blks));
                    }
                }
                Message::GetBlocks(blk_hashs) => {
                    debug!("Get blocks");
                    let mut res_blks: Vec<Block> = Vec::new();
                    for blk_hash in blk_hashs.iter() {
                        match self.blockchain.lock().unwrap().get_block(blk_hash) {
                            Some(blk) => {
                                res_blks.push(blk);
                            }
                            None => {}
                        }
                    }
                    if !res_blks.is_empty() {
                        peer.write(Message::Blocks(res_blks));
                    }
                }
                Message::Blocks(blks) => {
                    debug!("Comming blocks");
                    let mut new_hashs: Vec<H256> = Vec::new(); 
                    for blk_ref in blks.iter() {
                        //check whether the PoW is valid
                        let blk_hash: H256 = blk_ref.hash();
                        let current_difficulty: H256 = blk_ref.get_difficulty();
                        if blk_hash > current_difficulty {
                            debug!("reject block: difficulty not satisfied");
                            continue;
                        }
                        //check whether the hash is correct
                        if !Block::verify_hash(blk_ref) {
                            debug!("reject block: hash not valid");
                            continue;
                        }
                        //check whether the transactions inside are valid
                        let mut are_txs_valid = true;
                        for tx_ref in blk_ref.content.txs.data.iter() {
                            if !self.validate_tx(tx_ref, ValidationSource::FromBlock) {
                                are_txs_valid = false;
                                break;
                            } 
                        }
                        if !are_txs_valid {
                            debug!("reject block: some txs are not valid");
                            continue;
                        }
                        //check whether the parent exits
                        let parent_hash: H256 = blk_ref.get_parent();
                        let parent_blk: Option<Block> = self.blockchain.lock().unwrap().get_block(&parent_hash);
                        match parent_blk {
                            Some(blk) => {
                                //check whether the difficulty of the current block equals
                                //to that of the parent block
                                let parent_difficulty: H256 = blk.get_difficulty();
                                if parent_difficulty != current_difficulty {
                                    debug!("reject block: found varying difficulty");
                                    continue;
                                }
                            }
                            None => {
                                self.blk_buff.insert(hex::encode(&parent_hash.0), blk_ref.clone());
                                debug!("reject block: parent not found");
                                continue;
                            }
                        }

                        let mut inserted_blk: &Block = blk_ref;
                        let mut removed_buff: Vec<String> = Vec::new();
                        loop {
                            match self.blockchain.lock().unwrap().insert(inserted_blk) {
                                (true, _) => {
                                    //pod the transactions from mempool
                                    let deleted_txs = &inserted_blk.content.txs.data;
                                    let deleted_txs_hashs: Vec<H256> = deleted_txs
                                        .iter()
                                        .map(|x| x.hash())
                                        .collect();
                                    self.mempool.lock().unwrap().delete_txs(deleted_txs_hashs);
                                    let new_hash: H256 = inserted_blk.hash();
                                    new_hashs.push(new_hash.clone());
                                    let hash_str: String = hex::encode(&new_hash.0);
                                    match self.blk_buff.get(&hash_str) {
                                        Some(blk) => {
                                            inserted_blk = blk;
                                            removed_buff.push(hash_str);
                                        }
                                        None => {
                                            break;
                                        }
                                    }
                                }
                                (false, _) => {
                                    debug!("reject block: insert fail");
                                    break;
                                }
                            }
                        }
                        for item in removed_buff.iter() {
                            self.blk_buff.remove(item);
                        }
                    }
                    if !new_hashs.is_empty() {
                        self.server.broadcast(Message::NewBlockHashes(new_hashs));
                    }
                }
                Message::NewTransactionHashes(tx_hashs) => {
                    debug!("New transaction hashs");
                    let mut unreceived_txs: Vec<H256> = Vec::new();
                    for tx_hash in tx_hashs.iter() {
                        if self.mempool.lock().unwrap().check(tx_hash) {
                            continue;
                        }
                        if let Some(_) = self.blockchain
                                                .lock()
                                                .unwrap()
                                                .get_tx_in_longest_chain(tx_hash) {
                            continue;
                        }
                        unreceived_txs.push(tx_hash.clone());
                    }
                    if !unreceived_txs.is_empty() {
                        peer.write(Message::GetTransactions(unreceived_txs));
                    }
                }
                Message::GetTransactions(tx_hashs) => {
                    debug!("Get transactions");
                    let mut res_txs: Vec<Transaction> = Vec::new();
                    for tx_hash in tx_hashs.iter() {
                        //find tx in mempool
                        if let Some(tx) = self.mempool.lock().unwrap().get_tx(tx_hash) {
                            res_txs.push(tx);
                            continue;
                        }
                        //find tx in blockchain
                        if let Some(tx) = self.blockchain.lock().unwrap().get_tx_in_longest_chain(tx_hash) {
                            res_txs.push(tx);
                        }
                    }
                    if !res_txs.is_empty() {
                        peer.write(Message::Transactions(res_txs));
                    }
                }
                Message::Transactions(txs) => {
                    debug!("Comming transactions");
                    let mut new_tx_hashs: Vec<H256> = Vec::new();
                    for tx in txs.iter() {
                        let tx_hash: H256 = tx.hash();
                        //1.check whether the tx already exits
                        if self.mempool.lock().unwrap().check(&tx_hash) { 
                            continue;
                        }
                        if let Some(_) = self.blockchain
                                    .lock()
                                    .unwrap()
                                    .get_tx_in_longest_chain(&tx_hash) {
                            continue;
                        }
                        //2.validate the transaction
                        if !self.validate_tx(tx, ValidationSource::FromTransaction) {
                            continue;
                        }
                        new_tx_hashs.push(tx_hash);
                        self.mempool.lock().unwrap().insert(tx.clone());
                    }
                    if !new_tx_hashs.is_empty() {
                        self.server.broadcast(Message::NewTransactionHashes(new_tx_hashs));
                    }
                }
            }
        }
    }
}

//#[cfg(any(test,test_utilities))]
//struct TestMsgSender {
//    s: smol::channel::Sender<(Vec<u8>, peer::Handle)>
//}
//#[cfg(any(test,test_utilities))]
//impl TestMsgSender {
//    fn new() -> (TestMsgSender, smol::channel::Receiver<(Vec<u8>, peer::Handle)>) {
//        let (s,r) = smol::channel::unbounded();
//        (TestMsgSender {s}, r)
//    }
//
//    fn send(&self, msg: Message) -> PeerTestReceiver {
//        let bytes = bincode::serialize(&msg).unwrap();
//        let (handle, r) = peer::Handle::test_handle();
//        smol::block_on(self.s.send((bytes, handle))).unwrap();
//        r
//    }
//}
//#[cfg(any(test,test_utilities))]
///// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
//fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
//    let (server, server_receiver) = ServerHandle::new_for_test();
//    let (test_msg_sender, msg_chan) = TestMsgSender::new();
//    let difficulty: H256 = (&[255u8; 32]).into();
//    let blockchain: Arc<Mutex<Blockchain>> = Arc::new(Mutex::new(Blockchain::new(&difficulty)));
//    let worker = Worker::new(1, msg_chan, &server, &blockchain);
//    worker.start(); 
//    let res = (test_msg_sender, server_receiver, blockchain.lock().unwrap().all_blocks_in_longest_chain());
//    res
//}
//
//// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST
//
//#[cfg(test)]
//mod test {
//    use ntest::timeout;
//    use crate::types::block::generate_random_block;
//    use crate::types::hash::Hashable;
//
//    use super::super::message::Message;
//    use super::generate_test_worker_and_start;
//
//    #[test]
//    #[timeout(60000)]
//    fn reply_new_block_hashes() {
//        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
//        let random_block = generate_random_block(v.last().unwrap());
//        let mut peer_receiver = test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
//        let reply = peer_receiver.recv();
//        if let Message::GetBlocks(v) = reply {
//            assert_eq!(v, vec![random_block.hash()]);
//        } else {
//            panic!();
//        }
//    }
//    #[test]
//    #[timeout(60000)]
//    fn reply_get_blocks() {
//        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
//        let h = v.last().unwrap().clone();
//        let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
//        let reply = peer_receiver.recv();
//        if let Message::Blocks(v) = reply {
//            assert_eq!(1, v.len());
//            assert_eq!(h, v[0].hash())
//        } else {
//            panic!();
//        }
//    }
//    #[test]
//    #[timeout(60000)]
//    fn reply_blocks() {
//        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
//        let random_block = generate_random_block(v.last().unwrap());
//        let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
//        let reply = server_receiver.recv().unwrap();
//        if let Message::NewBlockHashes(v) = reply {
//            assert_eq!(v, vec![random_block.hash()]);
//        } else {
//            panic!();
//        }
//    }
//}
//
//// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
