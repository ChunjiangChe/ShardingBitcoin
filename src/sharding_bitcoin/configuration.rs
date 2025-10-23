use crate::types::hash::H256;


#[derive(Debug, Default, Clone)]
pub struct Configuration {
    pub block_diff: H256,
    pub order_diff: H256,
    pub block_size: usize,
    pub k: usize,
    pub shard_id: usize,
    pub node_id: usize,
    pub shard_num: usize,
    pub shard_size: usize,
    pub exper_number: usize,
    pub exper_iter: usize,
}

impl Configuration {
    pub fn new() -> Self {
        Configuration {
            block_diff: H256::default(),
            order_diff: H256::default(), 
            block_size: 0,
            k: 6,
            shard_id: 0,
            node_id: 0,
            shard_num: 0,
            shard_size: 0,
            exper_number: 0,
            exper_iter: 0,
        }
    }
}
