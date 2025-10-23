use crate::types::hash::H256;


#[derive(Debug, Default, Clone)]
pub struct Configuration {
    pub difficulty: H256,
    pub block_size: usize,
    pub k: usize,
    pub initial_balance: u32,
    pub user_size: usize,
    pub num_tx_recv: usize //the number of receivers of a transaction when generating txs
}

impl Configuration {
    pub fn new() -> Self {
        Configuration {
            difficulty: (&[255u8; 32]).into(),
            block_size: 1,
            k: 1,
            initial_balance: 1000,
            user_size: 3,
            num_tx_recv: 3,
        }
    }
}
