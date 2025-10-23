use crate::{
    optchain::{
        miner::{
            self as Miner,
            worker::Worker as MinerWorker,
        },
        configuration::Configuration,
        block::{
            versa_block::VersaBlock,
            proposer_block::ProposerBlock,
            availability_block::AvailabilityBlock,
        },
        network::{
            server as NetworkServer,
            worker::Worker as NetworkWorker,
        },
        blockchain::Blockchain,
        multichain::Multichain,
        mempool::Mempool,
        symbolpool::SymbolPool,
    },
    types::{
        hash::{
            H256,
        },
    },
};

use smol::channel;
use log::{error};
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


#[test]
fn test_miner() {
    let mut config = Configuration::new();

    let tx_diff_str = String::from("3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3cf3");
    let prop_diff_str = String::from("30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c30c3");
    let avai_diff_str = String::from("09c09c09c09c09c09c09c09c09c09c09c09c09c09c09c09c09c09c09c09c09c0");
    let in_avai_diff_str = String::from("0270270270270270270270270270270270270270270270270270270270270270");

    let tx_diff_bytes: [u8; 32] = decode_hex(tx_diff_str.as_str())
        .unwrap()
        .try_into().unwrap();
    let tx_diff_hash: H256 = tx_diff_bytes.into();

    let prop_diff_bytes: [u8; 32] = decode_hex(prop_diff_str.as_str())
        .unwrap()
        .try_into().unwrap();
    let prop_diff_hash: H256 = prop_diff_bytes.into();

    let avai_diff_bytes: [u8; 32] = decode_hex(avai_diff_str.as_str())
        .unwrap()
        .try_into().unwrap();
    let avai_diff_hash: H256 = avai_diff_bytes.into();

    let in_avai_diff_bytes: [u8; 32] = decode_hex(in_avai_diff_str.as_str())
        .unwrap()
        .try_into().unwrap();
    let in_avai_diff_hash: H256 = in_avai_diff_bytes.into();

    config.tx_diff = tx_diff_hash;
    config.prop_diff = prop_diff_hash;
    config.avai_diff = avai_diff_hash;
    config.in_avai_diff = in_avai_diff_hash;
    config.block_size = 4;
    config.prop_size = 4;
    config.avai_size = 4;
    config.ex_req_num = 1;
    config.in_req_num = 1;
    config.k = 1;
    config.shard_id = 0;
    config.node_id = 0;
    config.shard_num = 4;
    config.shard_size = 1;
    config.exper_number = 0;
    config.exper_iter = 1;

    // let api_port: u16 = api_addr.port();
    let prop_genesis_block = VersaBlock::PropBlock(ProposerBlock::default());
    let prop_chain = Blockchain::new(prop_genesis_block, &config);


    let avai_chains: Vec<Blockchain> = (0..config.shard_num)
        .into_iter()
        .map(|_| {
            let avai_genesis_block = VersaBlock::ExAvaiBlock(AvailabilityBlock::default());
            Blockchain::new(avai_genesis_block, &config)
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

    let p2p_addr = "127.0.0.1:6000"
        .parse::<net::SocketAddr>()
        .unwrap_or_else(|e| {
            error!("Error parsing P2P server address: {}", e);
            process::exit(1);
        });
    // start the p2p server
    let (server_ctx, server) = NetworkServer::new(p2p_addr, msg_tx, config.shard_id).unwrap();
    server_ctx.start().unwrap();
    
    let worker_ctx = NetworkWorker::new(
        2,
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

    miner.start(500000);
    
    let interval = time::Duration::from_micros(100000000);
    thread::sleep(interval);
    
    miner.exit();

    let prop_size = multichain.lock().unwrap().get_prop_size();
    println!("proposer chain size: {}", prop_size);
    println!("Availability chain sizes");
    for i in 0..config.shard_num {
        let avai_size = multichain
            .lock()
            .unwrap()
            .get_avai_size(i);
        println!("{}", avai_size);
    }

    multichain.lock().unwrap().print_proposer_chain();
    multichain.lock().unwrap().print_availability_chains();
}
