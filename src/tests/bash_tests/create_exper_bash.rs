#[allow(unused_imports)]
use std::{
    fs::File,
    io::{Write, Error},
};

#[allow(dead_code)]
const EXPER_NUMBER: usize = 28;
#[allow(dead_code)]
const SHARD_NUM: usize = 5;
#[allow(dead_code)]
const SHARD_SIZE: usize = 5;
#[allow(dead_code)]
const PROPAGATION_DELAY: usize = 100; // ms
#[allow(dead_code)]
const INCLUSIVE_DIFF: usize = 15;


#[test]
fn create_bash() {
    generate_exper_bash().unwrap();
}

#[test]
fn generate_exper_bash() -> Result<(), Error> {
    let basic_path = format!("./scripts/exper_{}/", EXPER_NUMBER);
    let back_to_root = String::from("#!/bin/bash\ncd ../..\n");
    //let total_diffs: Vec<usize> = vec![7, 11, 15, 23, 31]; //exper12_13
    //let total_diffs: Vec<usize> = vec![4, 13, 15, 20, 25]; //exper14_15
    let total_diffs: Vec<usize> = vec![15, 31, 63, 127, 255];
    //let total_diffs: Vec<usize> = vec![15, 15, 15, 15, 15, 15, 15]; //exper21-25
    //let total_diffs: Vec<usize> = vec![4, 4, 4, 4, 20]; //exper17-20
    for shard_id in 0..SHARD_NUM {
        let total_diff = total_diffs[shard_id];
        for node_index in 0..SHARD_SIZE {
            let mut add_node_delay: Vec<String> = vec![];
            for inter_node_index in 0..node_index {
                add_node_delay.push(format!(
                    "sudo ./scripts/network_simulation/start_network_node.sh delay {} {} {} {} {}\n",
                    shard_id+1, node_index+1,
                    shard_id+1, inter_node_index+1,
                    PROPAGATION_DELAY 
                ));
            }
            for past_shard in 0..shard_id {
                add_node_delay.push(format!(
                    "sudo ./scripts/network_simulation/start_network_node.sh delay {} {} {} {} {}\n",
                    shard_id+1, node_index+1,
                    past_shard+1, 1,
                    PROPAGATION_DELAY
                ));
                add_node_delay.push(format!(
                    "sudo ./scripts/network_simulation/start_network_node.sh delay {} {} {} {} {}\n",
                    shard_id+1, node_index+1,
                    past_shard+1, 2,
                    PROPAGATION_DELAY
                ));
            }
            let node_id = shard_id * SHARD_SIZE + node_index;
            let basic_cmd = format!(
                "sudo ip netns exec ramjet-s{}-n{} ./target/debug/bitcoin --p2p 10.0.{}.{}:6000 --api 10.0.{}.{}:7000", 
                shard_id+1, node_index+1,
                shard_id+1, (node_index+1)*2,
                shard_id+1, (node_index+1)*2,
            );
            let mut connect_nodes: Vec<String> = vec![];
            for inter_node_index in 0..node_index {
                connect_nodes.push(format!(
                    "-c 10.0.{}.{}:6000",
                    shard_id+1, (inter_node_index+1)*2,
                ));
            }
            for past_shard in 0..shard_id {
                connect_nodes.push(format!(
                    "-c 10.0.{}.2:6000", past_shard+1
                ));
                connect_nodes.push(format!(
                    "-c 10.0.{}.4:6000", past_shard+1
                ));
            }
            let shard_id_cmd = format!("--shardId {}", shard_id);
            let node_id_cmd = format!("--nodeId {}", node_id);
            let exper_number_cmd = format!("--experNumber {}", EXPER_NUMBER);
            let shard_num_cmd = format!("--shardNum {}", SHARD_NUM);
            let shard_size_cmd = format!("--shardSize {}", SHARD_SIZE);
            let total_diff_cmd = format!("--tDiff {}", total_diff);
            let inclusive_diff_cmd = format!("--iDiff {}", INCLUSIVE_DIFF);
            let mut final_cmd: String = back_to_root.clone();
            for delay in add_node_delay {
                final_cmd = format!("{}{}", final_cmd, delay);
            }
            final_cmd = format!("{}{}", final_cmd, basic_cmd);
            for connect_cmd in connect_nodes {
                final_cmd = format!("{} {}", final_cmd, connect_cmd);
            }
            final_cmd = format!("{} {}", final_cmd, shard_id_cmd);
            final_cmd = format!("{} {}", final_cmd, node_id_cmd);
            final_cmd = format!("{} {}", final_cmd, exper_number_cmd);
            final_cmd = format!("{} {}", final_cmd, shard_num_cmd);
            final_cmd = format!("{} {}", final_cmd, shard_size_cmd);
            final_cmd = format!("{} {}", final_cmd, total_diff_cmd);
            final_cmd = format!("{} {}", final_cmd, inclusive_diff_cmd);
            let path = format!("{}start_node_{}.sh", basic_path.clone(), node_id);
            let mut output = File::create(path)?;
            write!(output, "{}", final_cmd)?;
        }
    }
    Ok(())
}


