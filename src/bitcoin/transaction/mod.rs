pub mod generator;

use serde::{Serialize, Deserialize};
use ring::signature::{self, Ed25519KeyPair, Signature};
use crate::types::hash::{H256, Hashable};
use std::collections::{VecDeque, HashMap};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TxFlag{
    Initial,
    Normal,
}

impl Default for TxFlag {
    fn default() -> Self {
        TxFlag::Normal
    }
}

impl ToString for TxFlag {
    fn to_string(&self) -> String {
        match self {
            TxFlag::Initial => String::from("initial"),
            TxFlag::Normal => String::from("normal"),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Transaction {
    pub inputs: Vec<UtxoInput>,
    pub outputs: Vec<UtxoOutput>,
    pub flag: TxFlag,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UtxoInput {
    pub tx_hash: H256,
    pub value: u32,
    pub index: u32,
    pub sig_ref: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct UtxoOutput {
    pub receiver_addr: String,
    pub value: u32,
    pub public_key_ref: Vec<u8>,
}



impl Hashable for Transaction {
    fn hash(&self) -> H256 {
        let in_size: usize = self.inputs.len();
        let out_size: usize = self.outputs.len();

        let in_str: String = (0..in_size).map(|i|
            serde_json::to_string(&self.inputs[i]).unwrap()
        ).collect();
        let out_str: String = (0..out_size).map(|i|
            serde_json::to_string(&self.outputs[i]).unwrap()
        ).collect();
        
        let tx_str: String = format!("{}{}{}", 
            in_str, out_str, 
            self.flag.to_string());

        ring::digest::digest(&ring::digest::SHA256, tx_str.as_bytes()).into()
    }
}


impl Transaction {
    pub fn new() -> Self {
        let inputs: Vec<UtxoInput> = Vec::new();
        let outputs: Vec<UtxoOutput> = Vec::new();
        Transaction {
            inputs,
            outputs,
            flag: TxFlag::Normal,
        }
    }
    /// Create digital signature of a transaction
    pub fn sign(t: &Transaction, key: &Ed25519KeyPair) -> Signature {
        let serialized_tx: Vec<u8> = bincode::serialize(t).unwrap();
        key.sign(serialized_tx.as_slice())
    }

    /// Verify digital signature of a transaction, using public key instead of secret key
    pub fn verify(t: &Transaction, public_key_ref: &[u8], sig_ref: &[u8]) -> bool {
        let peer_public_key = signature::UnparsedPublicKey::new(
            &signature::ED25519, 
            public_key_ref
        );
        let serialized_tx: Vec<u8> = bincode::serialize(t).unwrap();
        let res = peer_public_key.verify(serialized_tx.as_slice(), sig_ref);
        match res {
            Ok(()) => {
                true
            }
            Err(_) => {
                false
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Mempool {
    block_size: usize,
    txs_map: HashMap<String, Transaction>, //the key is the hash of the tx, while value is the
    //exact value of tx
    txs_queue: VecDeque<H256>,
}


impl Mempool {
    pub fn new(block_size: usize) -> Self {
        Mempool {
            block_size,
            txs_map: HashMap::new(),
            txs_queue: VecDeque::new(),
        }
    }
    
    pub fn insert(&mut self, tx: Transaction) -> bool {
        let hash: H256 = tx.hash();
        let hash_str: String = hex::encode(&hash);
        if self.txs_map.contains_key(&hash_str) {
            false
        } else {
            self.txs_map.insert(hash_str, tx.clone());
            self.txs_queue.push_back(hash);
            true
        }
    }

    pub fn check(&self, hash: &H256) -> bool {
        self.txs_map.contains_key(&hex::encode(hash))
    }
        
    pub fn get_tx(&self, hash: &H256) -> Option<Transaction> {
        if self.check(hash) {
            Some(self.txs_map.get(&hex::encode(&hash)).unwrap().clone())
        } else {
            None
        }
    }

    pub fn get_all_tx_ref(&self) -> Vec<Transaction> {
        let mut res: Vec<Transaction> = Vec::new();
        for (_, val) in self.txs_map.iter() {
            res.push(val.clone());
        }
        res
    }

    pub fn delete_txs(&mut self, tx_hashs: Vec<H256>) -> bool {
        for tx_hash in tx_hashs.iter() {
            let tx_key: String = hex::encode(&tx_hash);
            self.txs_map.remove(&tx_key);
            self.txs_queue.retain(|x| x != tx_hash);
        }
        true
    }

    pub fn pop_txs(&mut self) -> Option<Vec<Transaction>> {
        if self.txs_queue.len() < self.block_size {
            None
        } else {
            let res: Vec<Transaction> = (0..self.block_size).map(|_| {
                let tx_hash: H256 = self.txs_queue.pop_front().unwrap();
                let tx_key: String = hex::encode(&tx_hash);
                let tx: Transaction = self.txs_map.get(&tx_key).unwrap().clone();
                self.txs_map.remove(&tx_key);
                tx
            }).collect(); 
            Some(res)
        }
    }

    pub fn get_all_txs(&self) -> Vec<H256> {
        let mut res: Vec<H256> = Vec::new();
        for (_, val) in self.txs_map.iter() {
            res.push(val.hash());
        }
        res
    }
}


//#[cfg(any(test, test_utilities))]
//pub fn generate_random_transaction() -> Transaction {
//    let inputs: Vec<UtxoInput> = Vec::new();
//    let outputs: Vec<UtxoOutput> = Vec::new();
//    Transaction {
//        tx_id,
//        inputs,
//        outputs,
//        flag: TxFlag::Normal,
//    }
//}
//
//
//
//// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::types::key_pair;
//    use ring::signature::KeyPair;
//
//    #[test]
//    fn sign_verify() {
//        let t = generate_random_transaction();
//        let key = key_pair::random();
//        let signature = sign(&t, &key);
//        assert!(verify(&t, key.public_key().as_ref(), signature.as_ref()));
//    }
//    #[test]
//    fn sign_verify_two() {
//        let t = generate_random_transaction();
//        let key = key_pair::random();
//        let signature = sign(&t, &key);
//        let key_2 = key_pair::random();
//        let t_2 = generate_random_transaction();
//        assert!(!verify(&t_2, key.public_key().as_ref(), signature.as_ref()));
//        assert!(!verify(&t, key_2.public_key().as_ref(), signature.as_ref()));
//    }
//
//    #[test]
//    fn sign_after_verify() {
//        let key = key_pair::random();
//        let inputs_1: Vec<UtxoInput> = Vec::new();
//        let mut outputs_1: Vec<UtxoOutput> = Vec::new();
//        let out = UtxoOutput {
//            receiver_addr: String::new(),
//            value: 10,
//            public_key_ref: key.public_key().as_ref().to_vec(),
//        };
//        outputs_1.push(out);
//        let tx1 = Transaction {
//            inputs: inputs_1,
//            outputs: outputs_1,
//            flag: TxFlag::Normal,
//        };
//        
//        let tx1_sig = sign(&tx1, &key);
//        let mut inputs_2: Vec<UtxoInput> = Vec::new();
//        let outputs_2: Vec<UtxoOutput> = Vec::new();
//        let input = UtxoInput {
//            tx_hash: tx1.hash(),
//            value: 10,
//            index: 0,
//            sig_ref: tx1_sig.as_ref().to_vec(),
//        };
//        inputs_2.push(input);
//        
//        let tx_id_2 = gen_random_tx_id();
//        let tx2 = Transaction {
//            inputs: inputs_2,
//            outputs: outputs_2,
//            flag: TxFlag::Normal,
//        };
//
//        //Now we gonna verify these two transactions
//        for input in tx2.inputs.iter() {
//            let sig_vec = input.sig_ref.clone();
//            let index: usize = input.index as usize;
//
//            let outs: Vec<UtxoOutput> = tx1.outputs.clone();
//            let output: &UtxoOutput = outs.get(index).unwrap();
//            let pub_key = output.public_key_ref.clone();
//            assert!(verify(&tx1, pub_key.as_slice(), sig_vec.as_slice()));
//        }
//        
//
//        
//    }
//}
//
//// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
