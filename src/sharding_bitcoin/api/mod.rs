use serde::Serialize;
use crate::{
    sharding_bitcoin::{
        multichain::Multichain,
        miner::Handle as MinerHandle,
        network::{
            server::Handle as NetworkServerHandle,
            message::Message,
        },
        mempool::Mempool,
        // validator::{
        //     Validator,
        // },
        configuration::Configuration,
        block::Info,
    },
    // types::{
    //     hash::{
    //         H256,
    //         Hashable,
    //     }
    // },
};

use log::{info};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread,
    fs::File,
    io::{Write},
};
use tiny_http::{
    Header,
    Response,
    Server as HTTPServer,
};
use url::Url;
use chrono::{DateTime, Local};

#[allow(dead_code)]
pub struct Server {
    handle: HTTPServer,
    miner: MinerHandle,
    network: NetworkServerHandle,
    multichain: Arc<Mutex<Multichain>>,
    mempool: Arc<Mutex<Mempool>>,
    config: Configuration,
}

#[derive(Serialize)]
struct ApiResponse {
    success: bool,
    message: String,
}

macro_rules! respond_result {
    ( $req:expr, $success:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let payload = ApiResponse {
            success: $success,
            message: $message.to_string(),
        };
        let resp = Response::from_string(serde_json::to_string_pretty(&payload).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}
macro_rules! respond_json {
    ( $req:expr, $message:expr ) => {{
        let content_type = "Content-Type: application/json".parse::<Header>().unwrap();
        let resp = Response::from_string(serde_json::to_string(&$message).unwrap())
            .with_header(content_type);
        $req.respond(resp).unwrap();
    }};
}

impl Server {
    pub fn start(
        addr: std::net::SocketAddr,
        miner: &MinerHandle,
        network: &NetworkServerHandle,
        multichain: &Arc<Mutex<Multichain>>,
        mempool: &Arc<Mutex<Mempool>>,
        config: &Configuration,
    ) {
        let handle = HTTPServer::http(&addr).unwrap();
        let server = Self {
            handle,
            miner: miner.clone(),
            network: network.clone(),
            multichain: Arc::clone(multichain),
            mempool: Arc::clone(mempool),
            config: config.clone(),
        };
        thread::spawn(move || {
            for req in server.handle.incoming_requests() {
                let miner = server.miner.clone();
                let network = server.network.clone();
                let multichain = Arc::clone(&server.multichain);
                // let multichain = server.multichain.clone();
                // let mempool = Arc::clone(&server.mempool);
                let config = server.config.clone();
                // let validator = Validator::new(
                //     &multichain,
                //     &mempool,
                //     &config,
                // );
                thread::spawn(move || {
                    // a valid url requires a base
                    let base_url = Url::parse(&format!("http://{}/", &addr)).unwrap();
                    let url = match base_url.join(req.url()) {
                        Ok(u) => u,
                        Err(e) => {
                            respond_result!(req, false, format!("error parsing url: {}", e));
                            return;
                        }
                    };
                    match url.path() {
                        "/miner/start" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let lambda = match params.get("lambda") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing lambda");
                                    return;
                                }
                            };
                            let lambda = match lambda.parse::<u64>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req,
                                        false,
                                        format!("error parsing lambda: {}", e)
                                    );
                                    return;
                                }
                            };
                            miner.start(lambda);
                            respond_result!(req, true, "ok");
                        }
                        "/miner/end" => {
                            miner.exit();
                            respond_result!(req, true, "ok");
                        }
                        "/network/ping" => {
                            network.broadcast(Message::Ping(String::from("Test ping")));
                            respond_result!(req, true, "ok");
                        }
                        // "/blockchain/log" => {
                        //     let path = format!("./log/optchain/exper_{}/iter_{}/{}.txt", config.exper_number, config.exper_iter, config.shard_id*config.shard_size+config.node_id);
                        //     let mut output = File::create(path).unwrap();
                        //     //record proposer chain
                        //     let prop_forking_rate = multichain
                        //         .lock()
                        //         .unwrap()
                        //         .get_order_forking_rate();
                        //     let _ = write!(output, "Proposer Chain forking rate {}:\n", prop_forking_rate);
                        //     let all_prop_blocks = multichain
                        //         .lock()
                        //         .unwrap()
                        //         .all_blocks_in_longest_proposer_chain();
                        //     for prop_hash in all_prop_blocks.iter() {
                        //         let prop_block = multichain
                        //             .lock()
                        //             .unwrap()
                        //             .get_proposer_block(&prop_hash)
                        //             .unwrap();
                        //         let timestamp = prop_block.get_timestamp();
                        //         let datetime: DateTime<Local> = timestamp.into();
                        //         let formatted_datetime = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                        //         let _ = write!(output, "proposer block {:?} created at {}\n", prop_hash, formatted_datetime);
                        //     }
                        //     let avai_forking_rate = multichain
                        //         .lock()
                        //         .unwrap()
                        //         .get_availability_forking_rate_by_shard(config.shard_id);
                        //     let _ = write!(output, "Availability Chain at shard {} forking rate {}:\n", config.shard_id, avai_forking_rate);
                        //     let all_avai_blocks = multichain
                        //         .lock()
                        //         .unwrap()
                        //         .all_blocks_in_longest_availability_chain_by_shard(config.shard_id);
                        //     for avai_hash in all_avai_blocks.iter() {
                        //         let avai_block = multichain
                        //             .lock()
                        //             .unwrap()
                        //             .get_avai_block_by_shard(&avai_hash, config.shard_id)
                        //             .unwrap();
                        //         if avai_block.get_shard_id() != config.shard_id {
                        //             continue;
                        //         }
                        //         let timestamp = avai_block.get_timestamp();
                        //         let datetime: DateTime<Local> = timestamp.into();
                        //         let formatted_datetime = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                        //         let _ = write!(output, "availability block {:?} created at {}\n", avai_hash, formatted_datetime);
                        //     }
                        //     respond_result!(req, true, "ok");
                        // }
                        "/blockchain/ordering-chain" => {
                            let v = multichain
                                .lock()
                                .unwrap()
                                .all_blocks_in_longest_order_chain();
                            let mut v_string: Vec<String> = v
                                .into_iter()
                                .map(|h| {
                                    let order_versa_block = multichain
                                        .lock()
                                        .unwrap()
                                        .get_order_block(&h)
                                        .unwrap();
                                    let timestamp = order_versa_block.get_timestamp();
                                    let datetime: DateTime<Local> = timestamp.into();
                                    let formatted_datetime = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

                                    let str = h.to_string();
                                    let left_slice = &str[0..3];
                                    let right_slice = &str[61..64];
                                    format!("{left_slice}..{right_slice}:{formatted_datetime}")
                                })
                                .collect();
                            let prop_forking_rate = multichain
                                .lock()
                                .unwrap()
                                .get_order_forking_rate();
                            v_string.push(format!("Ordering chain forking rate: {}", prop_forking_rate));
                            respond_json!(req, v_string);
                        }
                        "/blockchain/shard-chain" => {
                            let v = multichain
                                .lock()
                                .unwrap()
                                .all_blocks_in_longest_shard_chain_by_shard(config.shard_id);
                            let mut v_string: Vec<String> = v
                                .into_iter()
                                .map(|h| {
                                    let shard_versa_block = multichain
                                        .lock()
                                        .unwrap()
                                        .get_shard_block_by_shard(&h, config.shard_id)
                                        .unwrap();
                                    let timestamp = shard_versa_block.get_timestamp();
                                    let datetime: DateTime<Local> = timestamp.into();
                                    let formatted_datetime = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

                                    let str = h.to_string();
                                    let left_slice = &str[0..3];
                                    let right_slice = &str[61..64];
                                    format!("{left_slice}..{right_slice}:{formatted_datetime}")
                                })
                                .collect();
                            let shard_forking_rate = multichain
                                .lock()
                                .unwrap()
                                .get_shard_forking_rate_by_shard(config.shard_id);
                            v_string.push(format!("Shard chain at shard {} forking rate: {}", config.shard_id, shard_forking_rate));  
                            respond_json!(req, v_string);
                        }
                        "/blockchain/shard-chain-with-shard" => {
                            let params = url.query_pairs();
                            let params: HashMap<_, _> = params.into_owned().collect();
                            let shard_id = match params.get("shard-id") {
                                Some(v) => v,
                                None => {
                                    respond_result!(req, false, "missing shard id");
                                    return;
                                }
                            };
                            let shard_id = match shard_id.parse::<usize>() {
                                Ok(v) => v,
                                Err(e) => {
                                    respond_result!(
                                        req, 
                                        false, 
                                        format!("error parsing shard id: {}", e)
                                    );
                                    return;
                                }
                            };

                            let v = multichain
                                .lock()
                                .unwrap()
                                .all_blocks_in_longest_shard_chain_by_shard(shard_id);
                            let v_string: Vec<String> = v
                                .into_iter()
                                .map(|h| {
                                    let avai_versa_block = multichain
                                        .lock()
                                        .unwrap()
                                        .get_shard_block_by_shard(&h, shard_id)
                                        .unwrap();
                                    let timestamp = avai_versa_block.get_timestamp();
                                    let datetime: DateTime<Local> = timestamp.into();
                                    let formatted_datetime = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

                                    let str = h.to_string();
                                    let left_slice = &str[0..3];
                                    let right_slice = &str[61..64];
                                    format!("{left_slice}..{right_slice}:{formatted_datetime}")
                                })
                                .collect();
                            respond_json!(req, v_string);
                        }
                        _ => {
                            let content_type =
                                "Content-Type: application/json".parse::<Header>().unwrap();
                            let payload = ApiResponse {
                                success: false,
                                message: "endpoint not found".to_string(),
                            };
                            let resp = Response::from_string(
                                serde_json::to_string_pretty(&payload).unwrap(),
                            )
                            .with_header(content_type)
                            .with_status_code(404);
                            req.respond(resp).unwrap();
                        }
                    }
                });
            }
        });
        info!("API server listening at {}", &addr);
    }
}
