use crate::{
    // optchain::{
    //     transaction::*,
    // },
    optchain::{
        block::{
            BlockHeader,
            BlockContent,
            Info,
            transaction_block::TransactionBlock,
        },
        transaction::Transaction,
    },
    types::{
        random::Random,
        hash::{
            Hashable,
            H256,
        },
        merkle::MerkleTree,
    },
};
use std::time::SystemTime;
use rand::Rng;
use log::debug;


#[test]
fn test_block_header() {
    //test a random block header
    let random_block_header = BlockHeader::random();
    println!("random block header: {:?}", random_block_header);
    debug!("test debug for block header: {:?}", random_block_header);

    //test a hash function
    let hash = random_block_header.hash();
    let copy_block_header = random_block_header.clone();
    let copy_hash = copy_block_header.hash();
    assert_eq!(hash, copy_hash);

    let default_block_header = BlockHeader::default();
    let default_hash = default_block_header.hash();
    assert_ne!(hash, default_hash);

    // test a create function
    // If the usize number is too large, the program will 
    // panic (error get handled in BlockkHeader::create function)
    // let mut rng = rand::thread_rng();
    // let shard_id: usize = rng.gen();
    let shard_id: usize = 10;
    let prop_parent = H256::random();
    let inter_parent = H256::random();
    let global_parents = vec![(prop_parent.clone(), shard_id)];
    let prop_root = H256::random();
    let avai_root = H256::random();
    let cmt_root = H256::random();
    let timestamp = SystemTime::now();

    let block_header = BlockHeader::create(
        shard_id, 
        prop_parent.clone(),
        inter_parent.clone(),
        global_parents.clone(),
        prop_root.clone(),
        avai_root.clone(),
        cmt_root.clone(),
        timestamp.clone()
    );

    //test a info trait
    assert_eq!(shard_id, block_header.get_shard_id());
    assert_eq!(prop_parent, block_header.get_prop_parent());
    assert_eq!(inter_parent, block_header.get_inter_parent());
    assert_eq!(global_parents, block_header.get_global_parents());
    assert_eq!(prop_root, block_header.get_prop_root());
    assert_eq!(avai_root, block_header.get_avai_root());
    assert_eq!(cmt_root, block_header.get_cmt_root());
    assert_eq!(timestamp, block_header.get_timestamp());

    let _ = block_header.get_info_hash();

}

#[test]
fn test_transaction_block() {
    let block_header = BlockHeader::random();
    let nonce: u32 = 100;
    let transaction_block = TransactionBlock::new(
        block_header.clone(),
        nonce,
    );
    assert_eq!(block_header.get_shard_id(), transaction_block.get_shard_id());
    assert_eq!(block_header.get_prop_parent(), transaction_block.get_prop_parent());
    assert_eq!(block_header.get_inter_parent(), transaction_block.get_inter_parent());
    assert_eq!(block_header.get_global_parents(), transaction_block.get_global_parents());
    assert_eq!(block_header.get_prop_root(), transaction_block.get_prop_root());
    assert_eq!(block_header.get_avai_root(), transaction_block.get_avai_root());
    assert_eq!(block_header.get_cmt_root(), transaction_block.get_cmt_root());
    assert_eq!(block_header.get_timestamp(), transaction_block.get_timestamp());
    assert_eq!(block_header.get_info_hash(), transaction_block.get_info_hash());
    assert_eq!(nonce, transaction_block.get_nonce());
}

#[test]
fn test_block_content() {
    let prop_tx_set: Vec<TransactionBlock> = (0..7)
        .map(|_| TransactionBlock::random())
        .collect();

    let avai_tx_set: Vec<TransactionBlock> = (0..7)
        .map(|_| TransactionBlock::random())
        .collect();

    let block_size = 99;
    let txs: Vec<Transaction> = (0..block_size)
        .map(|_| Transaction::random())
        .collect();
    let merkle_txs = MerkleTree::<Transaction>::new(&txs);

    let block_content = BlockContent::create(
        MerkleTree::<TransactionBlock>::new(&prop_tx_set),
        MerkleTree::<TransactionBlock>::new(&avai_tx_set),
        merkle_txs
    );

    let mut rng = rand::thread_rng();
    let prop_index: usize = rng.gen_range(0..7);
    let avai_index: usize = rng.gen_range(0..7);
    let tx_index: usize = rng.gen_range(0..block_size);

    let prop_proof = block_content.get_prop_merkle_proof(prop_index);
    let avai_proof = block_content.get_avai_merkle_proof(avai_index);
    let tx_proof = block_content.get_tx_merkle_proof(tx_index);

    let prop_datum = prop_tx_set.get(prop_index).unwrap().hash();
    assert!(block_content.prop_merkle_prove(&prop_datum, &prop_proof, prop_index));

    let avai_datum = avai_tx_set.get(avai_index).unwrap().hash();
    assert!(block_content.avai_merkle_prove(&avai_datum, &avai_proof, avai_index));

    let tx_datum = txs.get(tx_index).unwrap().hash();
    assert!(block_content.tx_merkle_prove(&tx_datum, &tx_proof, tx_index));

}