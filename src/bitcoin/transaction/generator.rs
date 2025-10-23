use crate::{
    types::{
        key_pair,
        hash::{Hashable}
    },
    bitcoin::{
        network::{
            message::Message,
            server::Handle as ServerHandle,
        },
        blockchain::Blockchain,
        transaction::{
            Transaction,
            TxFlag,
            UtxoInput,
            UtxoOutput,
        },
        configuration::Configuration,
    },
};
use crossbeam::channel::{
    unbounded,
    Receiver,
    Sender,
    TryRecvError,
};
use std::{
    time,
    thread,
    sync::{Arc, Mutex},
    collections::{
        VecDeque,
        HashMap,
    },
};
use rand::{self, Rng};
use log::{info, debug};
use ring::signature::{Ed25519KeyPair, KeyPair};

enum ControlSignal {
    Start(u64),
    Stop,
    Exit,
}

enum OperatingState {
    Run(u64),
    Pause, 
    ShutDown,
}

pub struct Context {
    //channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
    users: Vec<String>,
    keys: HashMap<String, Ed25519KeyPair>,
    config: Configuration,
}

#[derive(Clone)]
pub struct Handle {
    //channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
}

pub fn new(server: &ServerHandle, blockchain: &Arc<Mutex<Blockchain>>,config: &Configuration) -> (Context, Handle) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    
    let mut rng = rand::thread_rng();    
    let users_keys: Vec<(String, Ed25519KeyPair)> = (0..config.user_size).map(|_| {
        let random_bytes: [u8; 32] = rng.gen();
        (hex::encode(&random_bytes), key_pair::random())
    }).collect();

    let mut users: Vec<String> = Vec::new();
    let mut keys: HashMap<String, Ed25519KeyPair> = HashMap::new();
    for item in users_keys {
        users.push(item.0.clone());
        keys.insert(item.0.clone(), item.1);
    }

    let ctx = Context{
        server: server.clone(),
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Pause,
        blockchain: Arc::clone(blockchain),
        users,
        keys,
        config: config.clone(),
    };
    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle)
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, theta: u64) {
        self.control_chan
            .send(ControlSignal::Start(theta))
            .unwrap();
    }
    pub fn stop(&self) {
        self.control_chan.send(ControlSignal::Stop).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("tx generator".to_string())
            .spawn(move ||{
                self.generator_loop();
            }).unwrap();
        info!("Generator initialized into pause mode");
    }

    #[allow(noop_method_call)]
    fn generator_loop(&mut self) {
        //generate the initial balance for each user
        let mut initial_txs: Vec<Transaction> = Vec::new();
        for user in self.users.iter() {
            let uin = UtxoInput::default();
            let pub_key = self.keys.get(user)
                .unwrap()
                .clone()
                .public_key()
                .as_ref()
                .to_vec();
            let uout = UtxoOutput {
                receiver_addr: user.clone(),
                value: self.config.initial_balance,
                public_key_ref: pub_key,
            };
            let inputs: Vec<UtxoInput> = vec![uin];
            let outputs: Vec<UtxoOutput> = vec![uout];
            let initial_tx = Transaction {
                inputs,
                outputs,
                flag: TxFlag::Initial,
            };
            initial_txs.push(initial_tx);
        }
        self.server.broadcast(Message::Transactions(initial_txs));


        //main generating loop
        loop {
            //check and react to control signals
            match self.operating_state {
                OperatingState::Pause => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Generator shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Generator starting in continuous mode with theta {}", i);
                            self.operating_state = OperatingState::Run(i);
                        }
                        ControlSignal::Stop => {}
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
                                info!("Generator shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Generator starting in continuous mode with theta {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Stop => {
                                self.operating_state = OperatingState::Pause;
                            }
                        }
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Generator control channel detached"),
                }
            };

            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = time::Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }

                //generating transactions\
                let mut rng = rand::thread_rng();
                let num_node = self.users.len();
                let payer_index: usize = rng.gen_range(0..num_node);
                let payer: String = self.users.get(payer_index).unwrap().clone();
                let receivers: Vec<String>  = (0..self.config.num_tx_recv).map(|_| {
                    let recv_index: usize = rng.gen_range(0..num_node);
                    self.users.get(recv_index).unwrap().clone()
                }).collect();
                let coins: Vec<usize> = (0..self.config.num_tx_recv).map(|_| 1).collect();
                if let Some(tx) = self.create_tx(payer, receivers, coins) {
                    let mut txs: Vec<Transaction> = Vec::new();
                    txs.push(tx.clone());
                    self.server.broadcast(Message::Transactions(txs));
                }
            }
        }
    }

    fn create_tx(&self, payer: String, receivers: Vec<String>, coins: Vec<usize>) -> Option<Transaction> {
        if receivers.len() != coins.len() {
            debug!("the size of receives and coins are not equal");
            return None;
        }


        let mut available_utxos: VecDeque<(Transaction, u32)> = VecDeque::new();
        let blockchain_states = self.blockchain.lock().unwrap().states.clone();
        let last_blk_hash = self.blockchain.lock().unwrap().longest_chain_hash.clone();
        let lastest_state = blockchain_states.get(&hex::encode(&last_blk_hash)).unwrap();
        for (key, tx) in lastest_state.iter() {
            let utxo_index = key.1;
            if tx.outputs.get(utxo_index as usize).unwrap().receiver_addr == payer {
                available_utxos.push_back((tx.clone(), utxo_index));
            }
        }

        let require_coins: usize = coins.iter().sum();

        let mut available_coins: Vec<(Transaction, u32)> = Vec::new();
        let mut curr_coins: usize = 0;
        loop {
            if let Some(utxo) = available_utxos.pop_front() {
                curr_coins += utxo.0.outputs.get(utxo.1 as usize).unwrap().value as usize;
                available_coins.push(utxo);
                if curr_coins >= require_coins {
                    break;
                } 
            } else {
                break;
            }
        }
        
        if curr_coins < require_coins {
            debug!("not enough utxo");
            return None;
        }
        let left_coins: usize = curr_coins - require_coins;

        let mut uins: Vec<UtxoInput> = Vec::new();
        let mut uouts: Vec<UtxoOutput> = Vec::new();

        for item in available_coins.iter() {
            let tx = &item.0;
            let index = item.1;
            let sig = Transaction::sign(&tx, self.keys.get(&payer).unwrap());
            let uin = UtxoInput {
                tx_hash: tx.hash(),
                value: tx.outputs[index as usize].value,
                index: index,
                sig_ref: sig.as_ref().to_vec()
            };
            uins.push(uin);
        }

        for i in 0..receivers.len() {
            let uout = UtxoOutput {
                receiver_addr: receivers[i].clone(),
                value: coins[i] as u32,
                public_key_ref: self.keys.get(&receivers[i])
                                    .unwrap()
                                    .public_key()
                                    .as_ref()
                                    .to_vec(),
            };
            uouts.push(uout);
        }

        if left_coins > 0 {
            let uout = UtxoOutput {
                receiver_addr: payer.clone(),
                value: left_coins as u32,
                public_key_ref: self.keys.get(&payer)
                                    .unwrap()
                                    .public_key()
                                    .as_ref()
                                    .to_vec(),
            };
            uouts.push(uout);
        }

        Some(Transaction {
            inputs: uins,
            outputs: uouts,
            flag: TxFlag::Normal,
        })
        
    }
}
