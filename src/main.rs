#[cfg(test)]
#[macro_use]
extern crate hex_literal;

pub mod types;
pub mod bitcoin;
pub mod sharding_bitcoin;
pub mod tests;
use clap::clap_app;
use env_logger::Env;
use log::LevelFilter;
// use log::{error, info};
// use std::{
//     num::ParseIntError,
// };

use crate::{
    // bitcoin::start as bitcoin_start,
    sharding_bitcoin::{
        start as sharding_bitcoin_start,
        // configuration::Configuration as OptchainConfiguration,
    },
};


fn main() {
    //run_bitcoin();
    // init logger
    
    // env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    env_logger::Builder::from_env(Env::default().default_filter_or("error"))
        .init();

    // log::error!("This will be printed");
    // log::warn!("This will NOT be printed");
    // log::info!("This will NOT be printed");
    // log::debug!("This will NOT be printed");
    // assert!(false);
    //let verbosity = matches.occurrences_of("verbose") as usize;
    //stderrlog::new().verbosity(verbosity).init().unwrap();
    let matches = clap_app!(Powchain =>
        (version: "0.1")
        (about: "PoW Blockchain client")
        (@subcommand sharding_bitcoin_start =>
            (about: "Run Optchain protocol")
            (@arg verbose:
                -v ... 
                "Increases the verbosity of logging")
            (@arg peer_addr: 
                --p2p [ADDR] 
                default_value("127.0.0.1:6000") 
                "Sets the IP address and the port of the P2P server")
            (@arg api_addr: 
                --api [ADDR] 
                default_value("127.0.0.1:7000") 
                "Sets the IP address and the port of the API server")
            (@arg known_peer: 
                -c --connect ... [PEER] 
                "Sets the peers to connect to at start")
            (@arg p2p_workers: 
                --("p2p-workers") [INT] 
                default_value("1") 
                "Sets the number of worker threads for P2P server")
            (@arg shard_id:
                --shardId [INT]
                "Sets the shard id of the node")
            (@arg node_id:
                --nodeId [INT]
                "Sets the id of the node")
            (@arg exper_number:
                --experNumber [INT]
                "Sets the number of experiment")
            (@arg exper_iter:
                --experIter [INT]
                "Sets the number of experiment")
            (@arg shard_num:
                --shardNum [INT]
                "Sets the number of shards")
            (@arg shard_size:
                --shardSize [INT]
                "Sets the size of shards")
            (@arg block_size:
                --blockSize [INT]
                "Sets the size of block")
            (@arg symbol_size:
                --symbolSize [INT]
                "Sets the size of a symbol")
            (@arg prop_size:
                --propSize [INT]
                "Sets the size of prop_tx_set for each proposer block")
            (@arg avai_size:
                --avaiSize [INT]
                "Sets the size of avai_tx_set for each availability block")
            (@arg ex_req_num:
                --eReq [INT]
                "the number of requested symbols for each exclusive transaction block")
            (@arg in_req_num:
                --iReq [INT]
                "the number of requested symbols for each inclusive transaction block")
            (@arg confirmation_depth:
                --k [INT]
                "Sets the confirmation_depth")
            (@arg tx_diff:
                --tDiff [STR]
                "Sets the difficulty of mining a transaction block")
            (@arg prop_diff:
                --pDiff [STR]
                "Sets the difficulty of mining a proposer block")
            (@arg avai_diff:
                --aDiff [STR]
                "Sets the difficulty of mining an availability block")
            (@arg in_avai_diff:
                --iDiff [STR]
                "Sets the difficulty of mining an inclusive availability block")
        )       
    )
    .get_matches();

    match matches.subcommand() {
        ("shardingbitcoin", Some(sub_m)) => {
            sharding_bitcoin_start(sub_m);
        }
        _ => unreachable!(), // clap ensures one of the subcommands is used
    }

    

}
