pub mod api;
pub mod block;
pub mod blockchain;
pub mod miner;
pub mod network;
pub mod transaction;
pub mod configuration;
// pub mod validator;
pub mod mempool;
pub mod multichain;

use crate::{
    types::{
        hash::{
            H256,
        },
    },
    sharding_bitcoin::{
        configuration::Configuration,
        mempool::Mempool,
        block::{
            BlockHeader,
            ShardBlock, 
            OrderBlock,
            versa_block::VersaBlock,
        },
        network::{
            server as NetworkServer,
            worker::Worker as NetworkWorker,
        },
        api::Server as ApiServer,
        miner::{
            self as Miner,
            worker::Worker as MinerWorker,
        },
        blockchain::Blockchain as Blockchain,
        multichain::Multichain,
    },
};


// use crossbeam::channel::{
//     unbounded,
//     Receiver,
//     Sender,
//     TryRecvError,
// };
// use clap::clap_app;
use smol::channel;
use log::{error, info};
use std::{
    net, 
    process, 
    thread, 
    time, 
    sync::{Arc, Mutex},
    num::ParseIntError,
    convert::TryInto,
};
// use env_logger::Env;

pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

pub fn start(sub_com: &clap::ArgMatches) {

    // parse p2p server address
    let p2p_addr = sub_com
        .value_of("peer_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P server address: {}", e);
            process::exit(1);
        });

    // parse api server address
    let api_addr = sub_com
        .value_of("api_addr")
        .unwrap()
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing API server address: {}", e);
            process::exit(1);
        });
    //parse the shard id
    let shard_id = sub_com
        .value_of("shard_id")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard id: {}", e);
            process::exit(1);
        });
    //parse the shard id
    let node_id = sub_com
        .value_of("node_id")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the node id: {}", e);
            process::exit(1);
        });
    //parse the shard id
    let exper_number = sub_com
        .value_of("exper_number")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the experiment number: {}", e);
            process::exit(1);
        });
    //parse the shard id
    let exper_iter = sub_com
        .value_of("exper_iter")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the experiment iter: {}", e);
            process::exit(1);
        });
    let shard_num = sub_com
        .value_of("shard_num")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard_num number: {}", e);
            process::exit(1);
        });
    let shard_size = sub_com
        .value_of("shard_size")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let block_size = sub_com
        .value_of("block_size")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the block size: {}", e);
            process::exit(1);
        });
    let confirmation_depth = sub_com
        .value_of("confirmation_depth")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing the confirmation depth: {}", e);
            process::exit(1);
        });
    let block_diff = sub_com
        .value_of("block_diff")
        .unwrap()
        .parse::<String>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let order_diff = sub_com
        .value_of("order_diff")
        .unwrap()
        .parse::<String>()
        .unwrap_or_else(|e| {
            error!("Error parsing the shard size: {}", e);
            process::exit(1);
        });
    let p2p_workers = sub_com
        .value_of("p2p_workers")
        .unwrap()
        .parse::<usize>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P workers: {}", e);
            process::exit(1);
        });
    
    
    let mut config = Configuration::new();
    let block_diff_bytes: [u8; 32] = decode_hex(block_diff.as_str())
        .unwrap()
        .try_into().unwrap();
    let order_diff_bytes: [u8; 32] = decode_hex(order_diff.as_str())
        .unwrap()
        .try_into().unwrap();
    let block_diff_hash: H256 = block_diff_bytes.into();
    let order_diff_hash: H256 = order_diff_bytes.into();

    config.block_diff = block_diff_hash;
    config.order_diff = order_diff_hash;
    config.block_size = block_size as usize;
    config.k = confirmation_depth as usize;
    config.shard_id = shard_id as usize;
    config.node_id = node_id as usize;
    config.exper_number = exper_number as usize;
    config.exper_iter = exper_iter as usize;
    config.shard_num = shard_num as usize;
    config.shard_size = shard_size as usize;
    // let shard_id = format!("{:x}", shard_id);
    info!("configuration: {:?}", config);

    // let api_port: u16 = api_addr.port();
    let order_genesis_block = OrderBlock::default();
    let ordering_chain = Blockchain::new(VersaBlock::OrderBlock(order_genesis_block), &config);


    let shard_chains: Vec<Blockchain> = (0..config.shard_num)
        .into_iter()
        .map(|i| {
            let mut header = BlockHeader::default();
            header.set_shard_id(i);
            let shard_block = ShardBlock::create(
                header,
                vec![],
                0,
            );
            let shard_genesis_block = VersaBlock::ShardBlock(shard_block);
            Blockchain::new(shard_genesis_block, &config)
        })
        .collect();
    // let chains_ref: Vec<&Arc<Mutex<Blockchain>>> = avai_chains
    //     .iter()
    //     .collect();
    let multichain = Arc::new(
        Mutex::new(
            Multichain::new(prop_chain, avai_chains, &config)
        )
    );

    let mempool = Arc::new(
        Mutex::new(
            Mempool::new(&config)
        )
    );

    let symbolpool = Arc::new(
        Mutex::new(
            SymbolPool::new(&config)
        )
    );

    // create channels between server and worker
    let (msg_tx, msg_rx) = channel::bounded(10000);

    // start the p2p server
    let (server_ctx, server) = NetworkServer::new(p2p_addr, msg_tx, config.shard_id).unwrap();
    server_ctx.start().unwrap();
    
    // start the worker
    let worker_ctx = NetworkWorker::new(
        p2p_workers,
        msg_rx,
        &server,
        &multichain,
        &mempool,
        &symbolpool,
        &config,
    );
    worker_ctx.start();

    // start the miner
    let (miner_ctx, miner, finished_block_chan) = Miner::new(&multichain, &mempool, &config);
    let miner_worker_ctx = MinerWorker::new(
        &server, 
        finished_block_chan, 
        &multichain,
        &mempool,
        &symbolpool,
        &config,
    );
    miner_ctx.start();
    miner_worker_ctx.start();


    // //start the sample monitor
    // let verifier_ctx = Verifier::new(
    //     &multichain, 
    //     &server, 
    //     &config,
    //     &symbolpool,
    // );
    // verifier_ctx.start();

    
    // connect to known peers
    if let Some(known_peers) = sub_com.values_of("known_peer") {
        let known_peers: Vec<String> = known_peers.map(|x| x.to_owned()).collect();
        let server = server.clone();
        thread::spawn(move || {
            for peer in known_peers {
                loop {
                    let addr = match peer.parse::<net::SocketAddr>() {
                        Ok(x) => x,
                        Err(e) => {
                            error!("Error parsing peer address {}: {}", &peer, e);
                            break;
                        }
                    };
                    match server.connect(addr) {
                        Ok(_) => {
                            info!("Connected to outgoing peer {}", &addr);
                            break;
                        }
                        Err(e) => {
                            error!(
                                "Error connecting to peer {}, retrying in one second: {}",
                                addr, e
                            );
                            thread::sleep(time::Duration::from_millis(1000));
                            continue;
                        }
                    }
                }
            }
        });

    }

    // start the API server
    ApiServer::start(
        api_addr,
        &miner,
        &server,
        &multichain,
        &mempool,
        &config,
    );

    loop {
        std::thread::park();
    }
}
