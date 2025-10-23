use serde::{Serialize, Deserialize};
use crate::{
    types::{
        hash::{H256, Hashable}, 
        merkle::MerkleTree
    },
    bitcoin::transaction::Transaction,
};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockHeader {
    parent: H256,
    nonce: u32,
    difficulty: H256,
    timestamp: SystemTime,
    merkle_root: H256,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockContent {
    pub txs: MerkleTree<Transaction>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub content: BlockContent,
    hash: H256,
}


impl Hashable for BlockHeader {
    fn hash(&self) -> H256 {
        let time_str: String = format!("{:?}", self.timestamp);
        let time_hash: H256 = ring::digest::digest(&ring::digest::SHA256, time_str.as_bytes()).into();
        let info_vec: Vec<H256> = vec![self.difficulty.clone(), time_hash, self.merkle_root.clone()];
        let info_hash: H256 = H256::multi_hash(&info_vec);
        let inner_hash: H256 = H256::chash(&self.parent, &info_hash);
        let outter_hash: H256 = H256::pow_hash(&inner_hash, self.nonce);
        outter_hash
    }
}

impl Hashable for Block {
    fn hash(&self) -> H256 {
        self.hash.clone()
    }
}

impl Block {
    pub fn get_parent(&self) -> H256 {
        self.header.parent.clone()
    }

    pub fn get_difficulty(&self) -> H256 {
        self.header.difficulty.clone()
    }

    pub fn set_difficulty(&mut self, difficulty: &H256) {
        self.header.difficulty = difficulty.clone();
    }

    pub fn new() -> Self {
        let parent: H256 = H256::from(&[0; 32]);
        let nonce: u32 = 0;
        let difficulty: H256 = H256::from(&[0; 32]);
        let zero_time = UNIX_EPOCH + Duration::new(0, 0);
        let timestamp: SystemTime = SystemTime::from(zero_time);
        let txs: Vec<Transaction> = Vec::new();
        let merkle_tree: MerkleTree<Transaction> = MerkleTree::new(txs.as_slice());

        let header: BlockHeader = BlockHeader {
            parent,
            nonce,
            difficulty,
            timestamp,
            merkle_root: merkle_tree.root(),
        };
        let content: BlockContent = BlockContent {
            txs: merkle_tree,
        };

        let hash: H256 = header.hash();

        Block {
            header,
            content,
            hash,
        }
    }

    pub fn verify_hash(blk: &Block) -> bool {
        blk.hash() == blk.header.hash()
    }

    pub fn construct(parent: H256, timestamp: SystemTime, difficulty: H256,
        txs: MerkleTree<Transaction>, nonce: u32) -> Block {
        let header: BlockHeader = BlockHeader {
            parent, 
            nonce,
            difficulty,
            timestamp,
            merkle_root: txs.root(),
        };

        let content: BlockContent = BlockContent {
            txs
        };

        let hash: H256 = header.hash();

        Block {
            header, 
            content,
            hash,
        }
    }
}

//#[cfg(any(test, test_utilities))]
//pub fn generate_random_block(parent: &H256) -> Block {
//    let nonce: u32 = rand::thread_rng().gen();
//    let difficulty: H256 = H256::from(&[0; 32]);
//    let timestamp: SystemTime = SystemTime::now();
//    let txs: Vec<Transaction> = Vec::new();
//    let merkle_tree: MerkleTree<Transaction> = MerkleTree::new(txs.as_slice());
//
//    let header: BlockHeader = BlockHeader {
//        parent: parent.clone(),
//        nonce,
//        difficulty,
//        timestamp,
//        merkle_root: merkle_tree.root(),
//    };
//    let content: BlockContent = BlockContent {
//        txs: merkle_tree,
//    };
//
//    let hash: H256 = header.hash();
//
//    Block {
//        header,
//        content,
//        hash,
//    }
//}
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::types::hash::H256;
//
//    #[test]
//    fn generate_block() {
//        let parent: H256 = H256::from(&[0; 32]);
//        let blk: Block = generate_random_block(&parent);
//        assert_eq!(blk.header.parent, parent);
//        assert_eq!(blk.header.merkle_root, H256::from(&[0; 32]));
//    }
//}
