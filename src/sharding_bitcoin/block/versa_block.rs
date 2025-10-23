use crate::{
    sharding_bitcoin::{
        block::{
            Info,
            ShardBlock,
            OrderBlock,
        },
    },
    types::hash::{H256, Hashable},
};
use std::{
    time::SystemTime,
};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub enum VersaBlock {   
    ShardBlock(ShardBlock),
    OrderBlock(OrderBlock),
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub enum VersaHash {
    ShardHash(H256),
    OrderHash(H256),
}

impl Default for VersaBlock {
    fn default() -> Self {
        VersaBlock::ShardBlock(ShardBlock::default())
    }
}

impl Hashable for VersaBlock {
    fn hash(&self) -> H256 {
        match self {
            VersaBlock::ShardBlock(shard_block) => shard_block.hash(),
            VersaBlock::OrderBlock(order_block) => order_block.hash(),
        }
    }
}

impl std::fmt::Debug for VersaBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            VersaBlock::ShardBlock(shard_block) => shard_block.fmt(f),
            VersaBlock::OrderBlock(order_block) => order_block.fmt(f),
        }
    }
}


impl VersaBlock {

    pub fn verify_hash(&self) -> bool {
        match self {
            VersaBlock::ShardBlock(shard_block) => shard_block.verify_hash(),
            VersaBlock::OrderBlock(order_block) => order_block.verify_hash(),
        }
    }

    pub fn get_shard_id(&self) -> usize {
        match self {
            VersaBlock::ShardBlock(shard_block) => shard_block.get_shard_id(),
            VersaBlock::OrderBlock(order_block) => order_block.get_shard_id(),
        }
    }

    pub fn get_parent(&self) -> H256 {
        match self {
            VersaBlock::ShardBlock(shard_block) => shard_block.get_shard_parent(),
            VersaBlock::OrderBlock(order_block) => order_block.get_order_parent(),
        }
    }


    pub fn get_merkle_root(&self) -> Option<H256> {
        match self {
            VersaBlock::ShardBlock(shard_block) => Some(shard_block.get_merkle_root()),
            VersaBlock::OrderBlock(_) => None,
        }
    }

    pub fn get_timestamp(&self) -> SystemTime {
        match self {
            VersaBlock::ShardBlock(shard_block) => shard_block.get_timestamp(),
            VersaBlock::OrderBlock(order_block) => order_block.get_timestamp(),
        }
    }

    pub fn get_info_hash(&self) -> Vec<H256> {
        match self {
            VersaBlock::ShardBlock(shard_block) => shard_block.get_info_hash(),
            VersaBlock::OrderBlock(order_block) => order_block.get_info_hash(),
        }
    }

}