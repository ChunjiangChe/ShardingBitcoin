use serde::{Serialize, Deserialize};

use crate::{
        types::{
        hash::H256, 
    },
    optchain::{
        block::{
            transaction_block::TransactionBlock,
            versa_block::{
                VersaBlock,
                VersaHash,
            }
        },
        symbolpool::{SymbolIndex, Symbol},
    }
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Ping(String),
    Pong(String),
    //Exclusive Block
    NewTxBlockHash(Vec<H256>),
    GetTxBlocks(Vec<H256>),
    TxBlocks(Vec<TransactionBlock>),
    //Versa Block
    NewBlockHash(Vec<VersaHash>),
    GetBlocks(Vec<VersaHash>),
    Blocks(Vec<VersaBlock>),
    //Data Availability Sample
    NewSymbols(Vec<SymbolIndex>),
    GetSymbols(Vec<SymbolIndex>), //(cmt_root: H256, tx_index)
    Symbols(Vec<Symbol>), 
    //key: block_hash, tx_index, value: (sample_index, sample) 
    //missing block
    // NewMissBlockHash((Vec<H256>, u32)),
}
