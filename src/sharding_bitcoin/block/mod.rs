pub mod versa_block;
use serde::{Serialize, Deserialize};
use crate::{
    types::{
        hash::{H256, Hashable}, 
        merkle::MerkleTree,
        random::Random,
    },
    sharding_bitcoin::{
        transaction::Transaction,
    },
};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use rand::Rng;

/*
------------
------------
------------
Block definition
------------
------------
------------
*/

#[derive(Clone, Serialize, Deserialize, Eq, Hash, PartialEq)]
pub struct BlockHeader {
    shard_id: u32, 
    order_parent: H256, 
    shard_parent: H256, 
    merkle_root: H256, 
    timestamp: SystemTime,
}
#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct BlockContent {
    txs: MerkleTree<Transaction>, 
    confirmed_shard_blocks: Vec<H256>, 
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct ShardBlock {
    header: BlockHeader,
    txs: MerkleTree<Transaction>, //a set of transactions
    hash: H256,
    nonce: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct OrderBlock {
    header: BlockHeader,
    confirmed_shard_blocks: Vec<H256>,
    hash: H256,
    nonce: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, Hash, PartialEq)]
pub struct Block {
    header: BlockHeader,
    content: BlockContent,
    hash: H256,
}

pub trait Content {

    fn get_txs(&self) -> Vec<Transaction>;
    // fn get_txs_ref(&self) -> &Vec<Transaction>;

    fn get_tx_merkle_tree(&self) -> MerkleTree<Transaction>;
    // fn get_txs_merkle_tree(&self) -> MerkleTree<Transaction>;
}

pub trait Info {
    fn get_shard_id(&self) -> usize;
    fn get_order_parent(&self) -> H256;
    fn get_shard_parent(&self) -> H256;
    fn get_merkle_root(&self) -> H256;
    fn get_timestamp(&self) -> SystemTime;
    fn get_info_hash(&self) -> Vec<H256>;
}


/*
------------
------------
------------
Block Header
------------
------------
------------
*/

impl Random for BlockHeader {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        let shard_id: u32 = rng.gen();
        let order_parent = H256::random();
        let shard_parent = H256::random();
        let merkle_root = H256::random();
        BlockHeader {
            shard_id,
            order_parent,
            shard_parent,
            merkle_root,
            timestamp: SystemTime::now(),
        }
    }
}

impl std::fmt::Debug for BlockHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "BlockHeader(shard_id: {}, hash: {:?})", self.shard_id, self.hash())
    }
}



impl Hashable for BlockHeader {
    fn hash(&self) -> H256 {
        let info_vec = self.get_info_hash(); 
        let info_hash: H256 = H256::multi_hash(&info_vec);
        let all_hashes: Vec<H256> = vec![
            info_hash, 
            self.order_parent.clone(), 
            self.shard_parent.clone(), 
            self.merkle_root.clone(),
            ];
        let all_hash: H256 = H256::multi_hash(&all_hashes);
        all_hash
    }
}

impl Default for BlockHeader {
    fn default() -> Self {
        BlockHeader {
            // parent: H256::default(),
            // nonce: 0 as u32,
            // difficulty: H256::default(),
            // merkle_root: H256::default(),
            shard_id: 0 as u32,
            order_parent: H256::default(),
            shard_parent: H256::default(),
            merkle_root: H256::default(),
            timestamp: SystemTime::from(UNIX_EPOCH + Duration::new(0,0)),
        }
    }
}


impl BlockHeader {
    pub fn create(
        shard_id: usize,
        order_parent: H256,
        shard_parent: H256,
        merkle_root: H256,
        // parent: H256, 
        // nonce: usize, 
        // difficulty: H256,  
        timestamp: SystemTime,
        // merkle_root: H256
    ) -> Self {
        let shard_id: u32 = u32::try_from(shard_id).expect("Shard id does not fit in u32!");
        BlockHeader {
            // parent, 
            // nonce: nonce as u32,
            // difficulty,
            shard_id,
            order_parent,
            shard_parent,
            merkle_root,
            timestamp,
            // merkle_root
        }
    }
    // pub fn get_mem_size(&self) -> usize {
    //     H256::get_mem_size() * (5+self.parent.len())
    //         + std::mem::size_of::<u32>()
    //         + std::mem::size_of::<SystemTime>()
    // }
    pub fn set_shard_id(&mut self, shard_id: usize) {
        self.shard_id = shard_id as u32;
    }
}

impl Info for BlockHeader {
    
    fn get_shard_id(&self) -> usize {
        self.shard_id as usize
    }
    fn get_order_parent(&self) -> H256 {
        self.order_parent.clone()
    }
    fn get_shard_parent(&self) -> H256 {
        self.shard_parent.clone()
    }
    fn get_merkle_root(&self) -> H256 {
        self.merkle_root.clone()
    }
    fn get_timestamp(&self) -> SystemTime {
        self.timestamp.clone()
    }
    
    fn get_info_hash(&self) -> Vec<H256> {
        let time_str = format!("{:?}", self.timestamp);
        let time_hash: H256 = ring::digest::digest(
            &ring::digest::SHA256,
            time_str.as_bytes()
        ).into();
        let shard_id_hash :H256 = ring::digest::digest(
            &ring::digest::SHA256,
            &self.shard_id.to_be_bytes()
        ).into();
        vec![
            // self.difficulty.clone(),
            time_hash,
            shard_id_hash,
            // self.merkle_root.clone(),
        ]
    }
}



impl Default for BlockContent {
    fn default() -> Self {
        BlockContent {
            txs: MerkleTree::<Transaction>::new(&[]),
            confirmed_shard_blocks: vec![],
        }
    }
}

impl Content for BlockContent {



    fn get_txs(&self) -> Vec<Transaction> {
        self.txs.data.clone()
    }
    // fn get_txs_ref(&self) -> &Vec<Transaction> {
    //     &self.txs.data
    // }

    fn get_tx_merkle_tree(&self) -> MerkleTree<Transaction> {
        self.txs.clone()
    }
    // fn get_txs_merkle_tree(&self) -> MerkleTree<Transaction> {
    //     self.txs.clone()
    // }
}

impl BlockContent {
    pub fn create(
        txs: Vec<Transaction>,
        confirmed_shard_blocks: Vec<H256>,
    ) -> Self {
        Self {
            txs: MerkleTree::<Transaction>::new(txs.as_slice()),
            confirmed_shard_blocks,
        }
    }
    

    
}

impl Default for Block {
    fn default() -> Self {
        Block {
            header: BlockHeader::default(),
            content: BlockContent::default(),
            hash: H256::default(),
        }
    }
}

impl Default for ShardBlock {
    fn default() -> Self {
        ShardBlock {
            header: BlockHeader::default(),
            txs: MerkleTree::<Transaction>::new(&[]),
            hash: H256::default(),
            nonce: 0,
        }
    }
}
impl Hashable for ShardBlock {
    fn hash(&self) -> H256 {
        H256::pow_hash(&self.header.hash(), self.nonce)
    }
}

impl Info for ShardBlock {
    fn get_shard_id(&self) -> usize {
        self.header.get_shard_id()
    }
    fn get_order_parent(&self) -> H256 {
        self.header.get_order_parent()
    }
    fn get_shard_parent(&self) -> H256 {
        self.header.get_shard_parent()
    }
    fn get_merkle_root(&self) -> H256 {
        self.header.get_merkle_root()
    }
    fn get_timestamp(&self) -> SystemTime {
        self.header.get_timestamp()
    }
    fn get_info_hash(&self) -> Vec<H256> {
        self.header.get_info_hash()
    }
}

impl ShardBlock {
    pub fn create(
        header: BlockHeader,
        txs: Vec<Transaction>,
        nonce: u32,
    ) -> Self {
        ShardBlock {
            hash: H256::pow_hash(&header.hash(), nonce),
            header,
            txs: MerkleTree::<Transaction>::new(txs.as_slice()),
            nonce,
        }
    }

    pub fn get_nonce(&self) -> u32 {
        self.nonce
    }

    pub fn verify_hash(&self) -> bool {
        H256::pow_hash(&self.header.hash(), self.nonce) == self.hash
    }
}

impl Content for ShardBlock {
    fn get_txs(&self) -> Vec<Transaction> {
        self.txs.data.clone()
    }
    fn get_tx_merkle_tree(&self) -> MerkleTree<Transaction> {
        self.txs.clone()   
    }
}

impl Default for OrderBlock {
    fn default() -> Self {
        OrderBlock {
            header: BlockHeader::default(),
            confirmed_shard_blocks: vec![],
            hash: H256::default(),
            nonce: 0,
        }
    }
}

impl Hashable for OrderBlock {
    fn hash(&self) -> H256 {
        H256::pow_hash(&self.header.hash(), self.nonce)
    }
}

impl Info for OrderBlock {
    fn get_shard_id(&self) -> usize {
        self.header.get_shard_id()
    }
    fn get_order_parent(&self) -> H256 {
        self.header.get_order_parent()
    }
    fn get_shard_parent(&self) -> H256 {
        self.header.get_shard_parent()
    }
    fn get_merkle_root(&self) -> H256 {
        self.header.get_merkle_root()
    }
    fn get_timestamp(&self) -> SystemTime {
        self.header.get_timestamp()
    }
    fn get_info_hash(&self) -> Vec<H256> {
        self.header.get_info_hash()
    }
}



impl OrderBlock {
    pub fn create(
        header: BlockHeader,
        confirmed_shard_blocks: Vec<H256>,
        nonce: u32,
    ) -> Self {
        OrderBlock {
            hash: H256::pow_hash(&header.hash(), nonce),
            header,
            confirmed_shard_blocks,
            nonce,
        }
    }

    pub fn get_nonce(&self) -> u32 {
        self.nonce
    }

    pub fn verify_hash(&self) -> bool {
        H256::pow_hash(&self.header.hash(), self.nonce) == self.hash
    }

    pub fn get_confirmed_shard_blocks(&self) -> Vec<H256> {
        self.confirmed_shard_blocks.clone()
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.hash.clone()
    }
}

impl Block {
    pub fn verify_hash(blk: &Block) -> bool {
        blk.hash() == blk.header.hash()
    }

    pub fn get_header(&self) -> BlockHeader {
        self.header.clone()
    }

    pub fn get_content(&self) -> BlockContent {
        self.content.clone()
    }

    // pub fn get_tx_merkle_proof(&self, tx_index: usize) -> Vec<H256> {
    //     self.content.get_tx_merkle_proof(tx_index)
    // }

    pub fn get_txs(&self) -> Vec<Transaction> {
        self.content.get_txs()
    }

    pub fn construct(
        shard_id: usize, 
        order_parent: H256, 
        shard_parent: H256,
        txs: Vec<Transaction>,
        confirmed_shard_blocks: Vec<H256>
    ) -> Block {

        // let txs = MerkleTree::<Transaction>::new(txs.as_slice());
        let merkle_tree = MerkleTree::<Transaction>::new(txs.as_slice());
        

        let header: BlockHeader = BlockHeader {
            shard_id: shard_id as u32, 
            order_parent,
            shard_parent,
            merkle_root: merkle_tree.root(),
            timestamp: SystemTime::now(),
        };

        let content: BlockContent = BlockContent {
            txs: merkle_tree,
            confirmed_shard_blocks,
        };

        let hash: H256 = header.hash();

        Block {
            header, 
            content,
            hash,
        }
    }
    pub fn get_confirmed_shard_blocks(&self) -> Vec<H256> {
        self.content.confirmed_shard_blocks.clone()
    }
    
}