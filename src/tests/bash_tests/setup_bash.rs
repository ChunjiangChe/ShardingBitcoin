#[allow(unused_imports)]
use std::{
    fs::{File, self},
    io::{Write, Error},
    env,
    num::ParseIntError,
    str::FromStr,
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OptchainConfigData {
    shard_num: usize, //how many shards totally
    shard_size: usize, //how many nodes in each shard
    block_size: usize, //the number of txs in each block
    symbol_size: usize, //the number of txs in each symbol
    prop_size: usize, //the number of tx_block in each proposal block
    avai_size: usize, //the number of tx_block in each availability block
    ex_req_num: usize, //the number of symbols requested for each in-shard block
    in_req_num: usize, //the number of symbols requested for each out-shard block  
    confirmation_depth: usize, //the confirmation depth, k
    mining_interval: usize, //the interval(ms) between two mining operations
    runtime: usize, //how long the experiment will run (10 s)
    tx_diff: String, //mining difficulty for tx blocks
    prop_diff: String, //mining difficulty for proposal blocks
    avai_diff: String, //mining difficulty for availability blocks
    in_avai_diff: String, //mining difficulty for inclusive availability blocks
    description: String, //the README of this experiment
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ManifoldchainConfigData {
    shard_num: usize, //how many shards totally
    shard_size: usize, //how many nodes in each shard
    block_size: usize, //the number of txs in each block
    confirmation_depth: usize, //the confirmation depth, k
    mining_interval: usize, //the interval(ms) between two mining operations
    tx_generation_interval: usize, //the interval(ms) between two tx generations
    runtime: usize, //how long the experiment will run (10 s)
    domestic_ratio: f64,
    inclusive_diff: String, //inclusive difficulty (all shard shares the same inclusive diff)
    exclusive_diffs: Vec<String>, //exclusive difficulties across all shards
    description: String, //the README of this experiment
}

#[test]
fn test_decode() {
    let diff = String::from("00000000ffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    let res = decode_hex(diff.as_str()).unwrap();
    println!("{:?}", res);
}

#[cfg(test)]
pub fn decode_hex(s: &str) -> Result<Vec<u8>, ParseIntError> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16))
        .collect()
}

#[derive(Debug, Clone, Copy)]
enum Protocol {
    Manifoldchain,
    Optchain,
}

#[derive(Debug, Clone)]
enum ConfigData {
    OptchainConfig(OptchainConfigData),
    ManifoldchainConfig(ManifoldchainConfigData),
}

impl FromStr for Protocol {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "manifoldchain" => Ok(Protocol::Manifoldchain),
            "optchain" => Ok(Protocol::Optchain),
            other => Err(format!("Unknown protocol: {}", other)),
        }
    }
}


#[test]
pub fn setup_script() {
    let args: Vec<String> = std::env::args().collect();
    println!("All args: {:?}", args);

    // The first arg is always the binary path, skip it.
    // After that, you’ll see "--protocol", "bitcoin", "--experNum", "1", "--experIter", "1"
    let mut protocol = String::new();
    let mut exper_num: Option<u32> = None;
    let mut exper_iter: Option<u32> = None;

    let mut iter = args.iter().skip(1); // skip program path
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--protocol" => {
                if let Some(val) = iter.next() {
                    protocol = val.clone();
                }
            }
            "--experNum" => {
                if let Some(val) = iter.next() {
                    exper_num = val.parse().ok();
                }
            }
            "--experIter" => {
                if let Some(val) = iter.next() {
                    exper_iter = val.parse().ok();
                }
            }
            _ => {}
        }
    }
    let exper_number = exper_num.expect("experNum not provided or invalid") as usize;
    let exper_iter = exper_iter.expect("experIter not provided or invalid") as usize;
    println!("Protocol = {}", protocol);
    println!("ExperNum = {}", exper_number);
    println!("ExperIter = {}", exper_iter);
    let protocol = Protocol::from_str(&protocol).expect("Invalid protocol");
    let protocol_location: String = match protocol.clone() {
        Protocol::Manifoldchain => String::from("manifoldchain"),
        Protocol::Optchain => String::from("optchain"),
    };
    let config_data = read_config(exper_number as usize, protocol.clone());
    generate_exper_bash(exper_number, exper_iter, protocol.clone(), config_data.clone()).unwrap();
    generate_start_bash(exper_number, exper_iter, protocol_location.clone(), config_data.clone());
    generate_start_nodes_bash(exper_number, exper_iter, protocol_location.clone(), config_data.clone());
    generate_end_bash(exper_number, exper_iter, protocol_location.clone(), config_data.clone());
}



#[cfg(test)]
fn read_config(exper_number: usize, protocol: Protocol) -> ConfigData {
    match protocol {
        Protocol::Manifoldchain => {
            let path = format!("./scripts/expers/manifoldchain/exper_{}/config.json", exper_number);
            let config_content = fs::read_to_string(path).expect("Couldn't find the file");
            let config_data: ManifoldchainConfigData = serde_json::from_str(&config_content).unwrap();
            ConfigData::ManifoldchainConfig(config_data)
        },
        Protocol::Optchain => {
            let path = format!("./scripts/expers/optchain/exper_{}/config.json", exper_number);
            let config_content = fs::read_to_string(path).expect("Couldn't find the file");
            let config_data: OptchainConfigData = serde_json::from_str(&config_content).unwrap();
            ConfigData::OptchainConfig(config_data)
        },    
    }
}

fn add_configuration(
    cmd: String, 
    exper_number: usize, 
    exper_iter: usize, 
    cf: ConfigData, 
    shard_id: usize, 
    node_id: usize
) -> String {
    match cf {
        ConfigData::OptchainConfig(config) => {
            let shard_id_cmd = format!("--shardId {}", shard_id);
            let node_id_cmd = format!("--nodeId {}", node_id);
            let exper_number_cmd = format!("--experNumber {}", exper_number);
            let exper_iter_cmd = format!("--experIter {}", exper_iter);
            let shard_num_cmd = format!("--shardNum {}", config.shard_num);
            let shard_size_cmd = format!("--shardSize {}", config.shard_size);
            let block_size_cmd = format!("--blockSize {}", config.block_size);
            let symbol_size_cmd = format!("--symbolSize {}", config.symbol_size);
            let prop_size_cmd = format!("--propSize {}", config.prop_size);
            let avai_size_cmd = format!("--avaiSize {}", config.avai_size);
            let ex_req_num_cmd = format!("--eReq {}", config.ex_req_num);
            let in_req_num_cmd = format!("--iReq {}", config.in_req_num);
            let confirmation_depth_cmd = format!("--k {}", config.confirmation_depth);
            let tx_diff_cmd = format!("--tDiff {}", config.tx_diff);
            let prop_diff_cmd = format!("--pDiff {}", config.prop_diff);
            let avai_diff_cmd = format!("--aDiff {}", config.avai_diff);
            let in_avai_diff_cmd = format!("--iDiff {}", config.in_avai_diff);
            let mut final_cmd = cmd;
            final_cmd = format!("{} {}", final_cmd, shard_id_cmd);
            final_cmd = format!("{} {}", final_cmd, node_id_cmd);
            final_cmd = format!("{} {}", final_cmd, exper_number_cmd);
            final_cmd = format!("{} {}", final_cmd, exper_iter_cmd);
            final_cmd = format!("{} {}", final_cmd, shard_num_cmd);
            final_cmd = format!("{} {}", final_cmd, shard_size_cmd);
            final_cmd = format!("{} {}", final_cmd, block_size_cmd);
            final_cmd = format!("{} {}", final_cmd, symbol_size_cmd);
            final_cmd = format!("{} {}", final_cmd, prop_size_cmd);
            final_cmd = format!("{} {}", final_cmd, avai_size_cmd);
            final_cmd = format!("{} {}", final_cmd, ex_req_num_cmd);
            final_cmd = format!("{} {}", final_cmd, in_req_num_cmd);
            final_cmd = format!("{} {}", final_cmd, confirmation_depth_cmd);
            final_cmd = format!("{} {}", final_cmd, tx_diff_cmd);
            final_cmd = format!("{} {}", final_cmd, prop_diff_cmd);
            final_cmd = format!("{} {}", final_cmd, avai_diff_cmd);
            final_cmd = format!("{} {}", final_cmd, in_avai_diff_cmd);
            final_cmd
        }
        ConfigData::ManifoldchainConfig(config) => {
            let exclusive_diff = config.exclusive_diffs[shard_id].clone();

            let shard_id_cmd = format!("--shardId {}", shard_id);
            let node_id_cmd = format!("--nodeId {}", node_id);
            let exper_number_cmd = format!("--experNumber {}", exper_number);
            let exper_iter_cmd = format!("--experIter {}", exper_iter);
            let shard_num_cmd = format!("--shardNum {}", config.shard_num);
            let shard_size_cmd = format!("--shardSize {}", config.shard_size);
            let block_size_cmd = format!("--blockSize {}", config.block_size);
            let confirmation_depth_cmd = format!("--k {}", config.confirmation_depth);
            let domestic_ratio_cmd = format!("--domesticRatio {}", config.domestic_ratio);
            let total_diff_cmd = format!("--eDiff {}", exclusive_diff);
            let inclusive_diff_cmd = format!("--iDiff {}", config.inclusive_diff);
            let mut final_cmd = cmd;
            final_cmd = format!("{} {}", final_cmd, shard_id_cmd);
            final_cmd = format!("{} {}", final_cmd, node_id_cmd);
            final_cmd = format!("{} {}", final_cmd, exper_number_cmd);
            final_cmd = format!("{} {}", final_cmd, exper_iter_cmd);
            final_cmd = format!("{} {}", final_cmd, shard_num_cmd);
            final_cmd = format!("{} {}", final_cmd, shard_size_cmd);
            final_cmd = format!("{} {}", final_cmd, block_size_cmd);
            final_cmd = format!("{} {}", final_cmd, confirmation_depth_cmd);
            final_cmd = format!("{} {}", final_cmd, domestic_ratio_cmd);
            final_cmd = format!("{} {}", final_cmd, total_diff_cmd);
            final_cmd = format!("{} {}", final_cmd, inclusive_diff_cmd);
            final_cmd
        }
    }
    
}

#[cfg(test)]
fn generate_exper_bash(exper_number: usize, exper_iter: usize, protocol: Protocol, config: ConfigData) -> Result<(), Error> {
    let protocol_location = match protocol {
        Protocol::Manifoldchain => "manifoldchain",
        Protocol::Optchain => "optchain",
    };
    let basic_path = format!("./scripts/expers/{}/exper_{}/", protocol_location, exper_number);
    let nodes_path = format!("{}nodes/", basic_path.clone());
    //create a dir to store the startup scripts for nodes
    fs::create_dir_all(nodes_path.clone()).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
    });
    let log_path = format!("./log/{}/exper_{}/iter_{}/exec_log/", protocol_location, exper_number, exper_iter);
    //create a dir to store the directory for logs
    fs::create_dir_all(log_path).unwrap_or_else(|why| {
        println!("! {:?}", why.kind());
    });
    let shard_num = match config.clone() {
        ConfigData::OptchainConfig(cfg) => cfg.shard_num,
        ConfigData::ManifoldchainConfig(cfg) => cfg.shard_num,
    };

    let shard_size = match config.clone() {
        ConfigData::OptchainConfig(cfg) => cfg.shard_size,
        ConfigData::ManifoldchainConfig(cfg) => cfg.shard_size,
    };

    let back_to_root = String::from("#!/bin/bash\ncd ../../../../../\n");
    for shard_id in 0..shard_num {
        for node_id in 0..shard_size {
            let basic_cmd = format!(
                "sudo ./target/debug/powchain {} --p2p 127.0.0.1:60{}{} --api 127.0.0.1:70{}{}", 
                protocol_location,
                shard_id, node_id,
                shard_id, node_id,
            );
            let mut connect_nodes: Vec<String> = vec![];
            for inter_node_index in 0..node_id {
                connect_nodes.push(format!(
                    "-c 127.0.0.1:60{}{}",
                    shard_id, inter_node_index,
                ));
            }
            for past_shard in 0..shard_id {
                for past_inter_node_index in 0..shard_size {    
                    connect_nodes.push(format!(
                        "-c 127.0.0.1:60{}{}", past_shard, past_inter_node_index
                    ));
                }
            }
            let mut final_cmd: String = back_to_root.clone();
            final_cmd = format!("{}{}", final_cmd, basic_cmd);
            for connect_cmd in connect_nodes {
                final_cmd = format!("{} {}", final_cmd, connect_cmd);
            }
            final_cmd = add_configuration(final_cmd, exper_number, exper_iter, config.clone(), shard_id, node_id);
            let path = format!("{}start_node_{}.sh", nodes_path.clone(), shard_id*shard_size+node_id);
            let mut output = File::create(path)?;
            write!(output, "{}", final_cmd)?;
        }
    }
    Ok(())
}

#[cfg(test)]
fn generate_start_bash(exper_number: usize, exper_iter: usize, protocol_location: String, config: ConfigData) {
    let mining_interval = match config.clone() {
        ConfigData::OptchainConfig(cfg) => cfg.mining_interval,
        ConfigData::ManifoldchainConfig(cfg) => cfg.mining_interval,
    };
    let runtime = match config.clone() {
        ConfigData::OptchainConfig(cfg) => cfg.runtime,
        ConfigData::ManifoldchainConfig(cfg) => cfg.runtime,
    };
    let shard_num = match config.clone() {
        ConfigData::OptchainConfig(cfg) => cfg.shard_num,
        ConfigData::ManifoldchainConfig(cfg) => cfg.shard_num,
    };
    let shard_size = match config.clone() {
        ConfigData::OptchainConfig(cfg) => cfg.shard_size,
        ConfigData::ManifoldchainConfig(cfg) => cfg.shard_size,
    };
    let start_tx_generator_or_not = match config.clone() {
        ConfigData::OptchainConfig(_) => String::from("echo \"skip\""),
        // ConfigData::ManifoldchainConfig(cfg) => format!("./start_tx_generator.sh {} {} {}", exper_number, exper_iter, cfg.tx_generation_interval.clone()),
        ConfigData::ManifoldchainConfig(_) => String::from("echo \"skip\""),
    };
    let content = format!(
"#!/bin/bash
shard_num={}
shard_size={}
mining_interval={}
runtime={}
iter={}
exper_number={}
sudo rm -r ../../../../DB/*
./start_nodes.sh
sleep 120
cd ../../../
for ((k=0; k<$shard_num; k++))
do
  for ((h=0; h<$shard_size; h++))
  do
    ./start_miner.sh $k $h $mining_interval 
    {}
  done
done
c=0
while [ $c -lt $runtime ]; do
  sleep 10
  c=$[$c+1]
  echo \"$c\"
  #log_count=$(( $c % 200 ))
  #if [ $log_count = 0 ]; then
      #for ((k=0; k<$shard_num; k++))
      #do
	      #for ((h=0; h<$shard_size; h++))
	      #do
		      #./ask_to_log.sh $k $h &
	      #done
      #done
  #fi
done
./end_node.sh $shard_num $shard_size
sleep 10", 
        shard_num, 
        shard_size, 
        mining_interval, 
        runtime,
        exper_iter,
        exper_number,
        start_tx_generator_or_not
    );
    let basic_path = format!("./scripts/expers/{}/exper_{}/", protocol_location, exper_number);
    let path = format!("{}start.sh", basic_path);
    let mut output = File::create(path).unwrap();
    write!(output, "{}", content).unwrap();
}

#[cfg(test)]
fn generate_start_nodes_bash(exper_number: usize, exper_iter: usize, protocol_location: String, config: ConfigData) {
    let shard_num = match config.clone() {
        ConfigData::OptchainConfig(cfg) => cfg.shard_num,
        ConfigData::ManifoldchainConfig(cfg) => cfg.shard_num,
    };  
    let shard_size = match config.clone() {
        ConfigData::OptchainConfig(cfg) => cfg.shard_size,
        ConfigData::ManifoldchainConfig(cfg) => cfg.shard_size,
    };
    let mut cmd = String::from("#!/bin/bash\n");
    let start_node_cmd = format!(
"for ((i=0; i<{}; i++))
do
  for ((j=0; j<{}; j++))
  do
    node_id=$[i*{}+j]
    cd nodes
    ./start_node_$node_id.sh 2>&1 | tee ../../../../../log/{}/exper_{}/iter_{}/exec_log/$node_id.log &
    cd ..
  done
done",
        shard_num,
        shard_size,
        shard_size,
        protocol_location,
        exper_number,
        exper_iter,
    );
    cmd = format!("{}{}", cmd, start_node_cmd);
    let basic_path = format!("./scripts/expers/{}/exper_{}/", protocol_location, exper_number);
    let path = format!("{}start_nodes.sh", basic_path);
    let mut output = File::create(path).unwrap();
    write!(output, "{}", cmd).unwrap();
}

#[cfg(test)]
fn generate_end_bash(exper_number: usize, exper_iter: usize, protocol_location: String, config: ConfigData) {
    let shard_num = match config.clone() {
        ConfigData::OptchainConfig(cfg) => cfg.shard_num,
        ConfigData::ManifoldchainConfig(cfg) => cfg.shard_num,
    };
    let shard_size = match config.clone() {
        ConfigData::OptchainConfig(cfg) => cfg.shard_size,
        ConfigData::ManifoldchainConfig(cfg) => cfg.shard_size,
    };
    let content = format!(
"#!/bin/bash

shard_num={}
shard_size={}
exper_number={}
iter={}

cd ../../../
./end_node.sh $shard_num $shard_size
sleep 10",
        shard_num, 
        shard_size,
        exper_number,
        exper_iter,
    );
    let basic_path = format!("./scripts/expers/{}/exper_{}/", protocol_location, exper_number);
    let path = format!("{}end.sh", basic_path);
    let mut output = File::create(path).unwrap();
    write!(output, "{}", content).unwrap();
}

