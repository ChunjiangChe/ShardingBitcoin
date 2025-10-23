use serde::{Serialize, Deserialize};

use crate::{
        types::{
        hash::H256, 
    },
    bitcoin::{
        block::Block,
        transaction::Transaction,
    }
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Ping(String),
    Pong(String),
    NewBlockHashes(Vec<H256>),
    GetBlocks(Vec<H256>),
    Blocks(Vec<Block>),
    NewTransactionHashes(Vec<H256>),
    GetTransactions(Vec<H256>),
    Transactions(Vec<Transaction>),
}
