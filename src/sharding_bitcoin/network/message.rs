use serde::{Serialize, Deserialize};

use crate::{
        types::{
        hash::H256, 
    },
    sharding_bitcoin::{
        block::{
            versa_block::{
                VersaBlock,
                VersaHash,
            }
        },
    }
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Ping(String),
    Pong(String),
    //Versa Block
    NewBlockHash(Vec<VersaHash>),
    GetBlocks(Vec<VersaHash>),
    Blocks(Vec<VersaBlock>),
    //key: block_hash, tx_index, value: (sample_index, sample) 
    //missing block
    // NewMissBlockHash((Vec<H256>, u32)),
}
