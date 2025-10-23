use crossbeam::channel::Receiver;
use log::{info};
use crate::{
    types::{
        hash::{H256, Hashable},
    },
    bitcoin::{
        block::Block,
        network::{
            server::Handle as ServerHandle,
            message::Message,
        },
        blockchain::Blockchain,
    }
};
use std::{thread, sync::{Arc, Mutex}};

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        blockchain: &Arc<Mutex<Blockchain>>,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            blockchain: Arc::clone(blockchain),
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let _block = self.finished_block_chan.recv().expect("Receive finished block error");
            // TODO for student: insert this finished block to blockchain, and broadcast this block hash
            self.blockchain.lock().unwrap().insert(&_block);
            let mut new_hashs: Vec<H256> = Vec::new();
            new_hashs.push(_block.hash());
            self.server.broadcast(Message::NewBlockHashes(new_hashs));
        }
    }
}
