pub mod worker;

use log::{info};
use crossbeam::channel::{
    unbounded, 
    Receiver, 
    Sender, 
    TryRecvError
};
use std::{
    time::{self}, 
    thread, 
    sync::{Arc, Mutex},
};
use crate::{        
    types::{
        hash::{H256, Hashable},
        random::Random,
    }, 
    optchain::{
        block::{
            Content,
            BlockContent,
            Block,
            transaction_block::TransactionBlock,
            proposer_block::ProposerBlock,
            availability_block::AvailabilityBlock,
            versa_block::VersaBlock,
        },
        multichain::Multichain,
        transaction::{Transaction},
        // validator::{
        //     Validator,
        // },
        configuration::Configuration,
        mempool::Mempool,
    },
};
use rand::Rng;

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Update, // update the block in mining, it may due to new blockchain tip or new transaction
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    finished_block_chan: Sender<MinerMessage>,
    multichain: Arc<Mutex<Multichain>>,
    mempool: Arc<Mutex<Mempool>>,
    // validator: Validator,
    config: Configuration,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(multichain: &Arc<Mutex<Multichain>>, 
    mempool: &Arc<Mutex<Mempool>>, 
    config: &Configuration) -> (Context, Handle, Receiver<MinerMessage>) 
{
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_block_sender, finished_block_receiver) = unbounded();

    // let validator = Validator::new(multichain, mempool, config);

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_block_chan: finished_block_sender,
        multichain: Arc::clone(multichain),
        mempool: Arc::clone(mempool),
        // validator,
        config: config.clone()
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_block_receiver)
}

pub enum MinerMessage {
    VersaBlk(VersaBlock),
    TxBlk((TransactionBlock, BlockContent)),
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

#[derive(Clone)]
pub enum DoubleSpentType {
    FromStaticState,
    FromDynamicState,
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn po_w(&self, block_hash: H256, nonce: u32) -> H256 {
        H256::pow_hash(&block_hash, nonce)
    }


    fn miner_loop(&mut self) {
        // main mining loop
        let mut pre_prop_parent = H256::default();
        let mut pre_inter_parent = H256::default();
        let mut pre_global_parents = H256::default();
        let mut pre_hybrid_block = Block::default();
        loop {
            // check and react to control signals
            // store the hash of parents in the previous round, 
            // if the parents does not change, just need to change the nonce
            // if the parents change, repackage the txs and generate the new consensus block
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Miner shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Miner starting in continuous mode with lambda {}", i);
                            self.operating_state = OperatingState::Run(i);
                        }
                        ControlSignal::Update => {
                            // in paused state, don't need to update
                        }
                    };
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        match signal {
                            ControlSignal::Exit => {
                                info!("Miner shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Miner starting in continuous mode with lambda {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                unimplemented!()
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }


            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
                let prop_parent = self.multichain
                    .lock()
                    .unwrap()
                    .get_highest_prop_block();
                let inter_parent = self.multichain
                    .lock()
                    .unwrap()
                    .get_highest_avai_block(self.config.shard_id);
                let global_parents = self.multichain
                    .lock()
                    .unwrap()
                    .get_all_highest_avai_blocks();
                let global_parents_hash = H256::multi_hash(&(
                    global_parents.iter()
                                  .map(|(hash, _)| hash.clone())
                                  .collect()
                ));
                // let mut txs: Vec<Transaction> = vec![];
                let txs: Vec<Vec<Transaction>>;
                //check if parents have been change
                if prop_parent != pre_prop_parent ||
                    inter_parent != pre_inter_parent ||
                    global_parents_hash != pre_global_parents {
                    
                    // randomly generate a constant number of transactions
                    txs = (0..self.config.num_symbol_per_block)
                                .into_iter()
                                .map(|_| {
                                    let sym: Vec<Transaction> = (0..self.config.symbol_size)
                                        .into_iter()
                                        .map(|_| Transaction::random())
                                        .collect();
                                    sym
                                }).collect();

                    let prop_tx_set = match self.mempool
                        .lock()
                        .unwrap()
                        .get_tx_blocks(self.config.prop_size) 
                    {
                        Ok(enough_set) => enough_set,
                        Err(insufficient_set) => insufficient_set,
                    };

                    let avai_tx_set = match self.multichain
                        .lock()
                        .unwrap()
                        .get_avai_tx_blocks(self.config.avai_size) 
                    {
                        Ok(enough_set) => enough_set,
                        Err(insufficient_set) => insufficient_set,
                    };
                    
                    // let mut supposed_global_parents = global_parents.clone();
                    // supposed_global_parents.retain(|x| x.1 != self.config.shard_id );
                    // supposed_global_parents.push((vec![last_blk_hash.clone()], self.config.shard_id));
                    let hybrid_block = Block::construct(
                        self.config.shard_id,
                        prop_parent.clone(),
                        inter_parent.clone(),
                        global_parents,
                        prop_tx_set,
                        avai_tx_set,
                        txs,
                    );
                    //info!("mines a block with parent {:?} of state size: {}", last_blk_hash, last_state.len());
                    //update related information
                    pre_prop_parent = prop_parent;
                    pre_inter_parent = inter_parent;
                    pre_global_parents = global_parents_hash;
                    pre_hybrid_block = hybrid_block;
                }
                
                let nonce: u32 = rand::thread_rng().gen();
                let hash_val = self.po_w(pre_hybrid_block.hash(), nonce);
                //info!("block hash: {:?}", hash_val);
                let tx_diff = self.config.tx_diff;
                // let mut supposed_global_parents = global_parents.clone();
                // supposed_global_parents.retain(|x| x.1 != self.config.shard_id );
                // supposed_global_parents.push((vec![last_blk_hash.clone()], self.config.shard_id));
                if hash_val <= tx_diff {
                    if hash_val <= self.config.prop_diff {
                        if hash_val <= self.config.avai_diff {
                            if hash_val <= self.config.in_avai_diff {
                                info!("mine an inclusive availability block {:?} in shard {}", hash_val, self.config.shard_id);
                                let in_block = VersaBlock::InAvaiBlock(AvailabilityBlock::new(    
                                    pre_hybrid_block.get_header(),
                                    nonce,
                                    pre_hybrid_block.get_avai_merkle_tree(),
                                ));
                                self.finished_block_chan
                                    .send(MinerMessage::VersaBlk(in_block))
                                    .unwrap();
                            } else {
                                info!("mine an exclusive availability block {:?} in shard {}", hash_val, self.config.shard_id);
                                let ex_block = VersaBlock::ExAvaiBlock(AvailabilityBlock::new(    
                                    pre_hybrid_block.get_header(),
                                    nonce,
                                    pre_hybrid_block.get_avai_merkle_tree(),
                                ));
                                self.finished_block_chan
                                    .send(MinerMessage::VersaBlk(ex_block))
                                    .unwrap();
                            }
                        } else {
                            info!("mine a proposer block {:?} in shard {}", hash_val, self.config.shard_id);
                            let prop_block = VersaBlock::PropBlock(ProposerBlock::new(    
                                pre_hybrid_block.get_header(),
                                nonce,
                                pre_hybrid_block.get_prop_merkle_tree(),
                            ));
                            self.finished_block_chan
                                .send(MinerMessage::VersaBlk(prop_block))
                                .unwrap();
                            //leave the job of inserting new blocks to the workers
                        }
                    } else {
                        info!("mine a transaction block {:?} in shard {}", hash_val, self.config.shard_id);
                        let tx_block = TransactionBlock::new(
                            pre_hybrid_block.get_header(),
                            nonce
                        );
                        self.finished_block_chan
                            .send(MinerMessage::TxBlk((tx_block, pre_hybrid_block.get_content())))
                            .unwrap();
                    }
                    pre_prop_parent = H256::default();
                    pre_inter_parent = H256::default();
                    pre_global_parents = H256::default();
                    pre_hybrid_block = Block::default();
                } else {
                    //no block is mined
                }

                
            }
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST


