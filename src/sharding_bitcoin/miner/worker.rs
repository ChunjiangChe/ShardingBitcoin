use crossbeam::channel::Receiver;
use log::{info};
use crate::{
    sharding_bitcoin::{
        block::{
            Info,
            // Content,
            versa_block::{
                VersaBlock,
                VersaHash,
            }
        },
        network::{
            server::Handle as ServerHandle,
            message::Message,
        },
        multichain::Multichain,
        miner::MinerMessage,
        configuration::Configuration,
        mempool::Mempool,
    }
};
use std::{
    thread, 
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<MinerMessage>,
    multichain: Arc<Mutex<Multichain>>,
    mempool: Arc<Mutex<Mempool>>,
    config: Configuration,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<MinerMessage>,
        multichain: &Arc<Mutex<Multichain>>,
        mempool: &Arc<Mutex<Mempool>>,
        config: &Configuration,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            multichain: Arc::clone(multichain),
            mempool: Arc::clone(mempool),
            config: config.clone(),
        }
    }

    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&mut self) {
        loop {
            let message = self.finished_block_chan
                .recv()
                .expect("Receive finished block error");
             
            match message {
                MinerMessage::VersaBlk(versa_block) => {
                    match versa_block.clone() {
                        VersaBlock::ShardBlock(shard_block) => {
                            //exclusive avaialbility block
                            let shard_parent = shard_block.get_shard_parent();
                            match self.multichain
                                .lock()
                                .unwrap()
                                .insert_block_with_parent(
                                versa_block.clone(),
                                &VersaHash::ShardHash(shard_parent)
                            ) {
                                Ok(_) => {}
                                Err(e) => {
                                    info!("inserting myself fail: {}", e);
                                }
                            }
                            self.server.broadcast(
                                Message::Blocks(vec![versa_block])
                            );
                        }
                        VersaBlock::OrderBlock(order_block) => {
                            let order_parent = order_block.get_order_parent();
                            match self.multichain
                                .lock()
                                .unwrap()
                                .insert_block_with_parent(
                                versa_block.clone(),
                                &VersaHash::OrderHash(order_parent)
                            ) {
                                Ok(_) => {}
                                Err(e) => {
                                    info!("inserting myself fail: {}", e);
                                }
                            }
                            self.server.broadcast(
                                Message::Blocks(vec![versa_block])
                            );
                        }
                    }
                }    
            }

        }
    }
}
