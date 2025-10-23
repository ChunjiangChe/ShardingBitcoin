use crate::{
    optchain::{
        blockchain::Blockchain,
        configuration::Configuration,
        block::{
            BlockHeader,
            transaction_block::TransactionBlock,
            proposer_block::ProposerBlock,
            versa_block::VersaBlock,
        },
    },
    types::{
        merkle::MerkleTree,
        hash::{H256, Hashable},
        random::Random,
    },
};
// use log::debug;
use std::time::SystemTime;

#[test]
fn test_blockchain() {
    //test generating a new blockchain
    let config = Configuration::new();
    
    let prop_genesis_block = VersaBlock::PropBlock(ProposerBlock::default());
    let mut blockchain = Blockchain::new(prop_genesis_block, &config);

    let parent = blockchain.tip();
    // println!("{:?}", parent);


    let prop_tx_set: Vec<TransactionBlock> = (0..10)
        .map(|_| TransactionBlock::random())
        .collect();
    let prop_tx_set = MerkleTree::<TransactionBlock>::new(&prop_tx_set);

    //test inserting a block
    let block_header = BlockHeader::create(
        0, //shard_id
        parent.clone(), //prop_parent,
        H256::default(), //inter_parent,
        vec![], //global_parent,
        prop_tx_set.root(), //prop_root,
        H256::default(), //avai_root
        H256::default(), //cmt_root
        SystemTime::now(), //timestamp
    );
    let prop_block = ProposerBlock::new(
        block_header, //header
        0, //nonce
        prop_tx_set, //prop_tx_set
    );

    let versa_block = VersaBlock::PropBlock(prop_block);
    blockchain.insert_block_with_parent(versa_block.clone(), &parent).expect("insert failure");

    //insert a second block
    let prop_tx_set_2: Vec<TransactionBlock> = (0..10)
        .map(|_| TransactionBlock::random())
        .collect();
    let prop_tx_set_2 = MerkleTree::<TransactionBlock>::new(&prop_tx_set_2);
    
    let block_header_2 = BlockHeader::create(
        0, //shard_id
        versa_block.hash(), //prop_parent,
        H256::default(), //inter_parent,
        vec![], //global_parent,
        prop_tx_set_2.root(), //prop_root,
        H256::default(), //avai_root
        H256::default(), //cmt_root
        SystemTime::now(), //timestamp
    );
    let prop_block_2 = ProposerBlock::new(
        block_header_2, //header
        0, //nonce
        prop_tx_set_2, //prop_tx_set
    );
    let versa_block_2 = VersaBlock::PropBlock(prop_block_2);
    blockchain.insert_block_with_parent(versa_block_2.clone(), &(versa_block.hash())).expect("insert failure");

    //insert a third block
    let prop_tx_set_3: Vec<TransactionBlock> = (0..10)
        .map(|_| TransactionBlock::random())
        .collect();
    let prop_tx_set_3 = MerkleTree::<TransactionBlock>::new(&prop_tx_set_3);
    
    let block_header_3 = BlockHeader::create(
        0, //shard_id
        versa_block.hash(), //prop_parent,
        H256::default(), //inter_parent,
        vec![], //global_parent,
        prop_tx_set_3.root(), //prop_root,
        H256::default(), //avai_root
        H256::default(), //cmt_root
        SystemTime::now(), //timestamp
    );
    let prop_block_3 = ProposerBlock::new(
        block_header_3, //header
        0, //nonce
        prop_tx_set_3, //prop_tx_set
    );
    let versa_block_3 = VersaBlock::PropBlock(prop_block_3);
    blockchain.insert_block_with_parent(versa_block_3.clone(), &(versa_block.hash())).expect("insert failure");

    //test all_blocks_in_longest_chain
    let longest_path_hashes = blockchain.all_blocks_in_longest_chain();
    assert_eq!(longest_path_hashes.len(), 3);
    assert_eq!(longest_path_hashes.get(1).unwrap().clone(), versa_block.hash());
    assert_eq!(longest_path_hashes.get(2).unwrap().clone(), versa_block_2.hash());

    let r_versa_block = blockchain.get_block(&(versa_block.hash())).expect("Block does not exist.");
    let r_versa_block_2 = blockchain.get_block(&(versa_block_2.hash())).expect("Block does not exist.");
    let r_versa_block_3 = blockchain.get_block(&(versa_block_3.hash())).expect("Block does not exist.");

    assert_eq!(r_versa_block.hash(), versa_block.hash());
    assert_eq!(r_versa_block_2.hash(), versa_block_2.hash());
    assert_eq!(r_versa_block_3.hash(), versa_block_3.hash());

    // assert!(blockchain.is_block_in_longest_chain(&(versa_block.hash())));
    // assert!(blockchain.is_block_in_longest_chain(&(versa_block_2.hash())));
    // assert!(blockchain.is_block_in_longest_chain(&(versa_block_3.hash())));
}