use crate::{
    optchain::{
        multichain::Multichain,
        configuration::Configuration,
        block::{
            versa_block::{VersaBlock, VersaHash},
            proposer_block::ProposerBlock,
            availability_block::AvailabilityBlock,
            transaction_block::TransactionBlock,
            BlockHeader,
            BlockContent,
        },
        blockchain::Blockchain,
        transaction::Transaction,
    },
    types::{
        hash::{
            Hashable, H256,
        },
        merkle::MerkleTree,
        random::Random,
    },
};
use std::collections::HashSet;
use std::time::{SystemTime};

#[test]
fn test_multichain() {
    let mut config = Configuration::new();
    config.shard_id = 0;
    config.shard_num = 10;
    let other_shard_id = 2;
    
    let prop_genesis_block = VersaBlock::PropBlock(ProposerBlock::default());
    let prop_chain = Blockchain::new(prop_genesis_block.clone(), &config);

    let avai_chains: Vec<Blockchain> = (0..config.shard_num)
        .into_iter()
        .map(|_| {
            let avai_genesis_block = VersaBlock::ExAvaiBlock(AvailabilityBlock::default());
            Blockchain::new(avai_genesis_block, &config)
        })
        .collect();
    
    let avai_genesis_block_current_shard = avai_chains.get(config.shard_id).unwrap().get_genesis_block();
    let avai_genesis_block_other_shard = avai_chains.get(other_shard_id).unwrap().get_genesis_block();
    let mut multichain = Multichain::new(prop_chain, avai_chains, &config);

    let unreferred_cmts = multichain.get_unreferred_cmt(&prop_genesis_block.hash());
    assert_eq!(unreferred_cmts.len(), 0);
    
    //insert a new proposer block
    //create some random cmts
    let prop_tx_set: Vec<TransactionBlock> = (0..7)
        .map(|_| {
            let mut h = BlockHeader::random();
            h.set_shard_id(config.shard_id);
            TransactionBlock::new(h, 0)
        })
        .collect();
    let prop_tx_tree: MerkleTree<TransactionBlock> = MerkleTree::<TransactionBlock>::new(&prop_tx_set);
    //create a block body
    let _prop_block_content = BlockContent::create(
        prop_tx_tree.clone(),
        MerkleTree::<TransactionBlock>::new(&(vec![])),
        MerkleTree::<Transaction>::new(&(vec![]))
    );

    let prop_block_header = BlockHeader::create(
        config.shard_id, //shard_id
        prop_genesis_block.hash(), //prop_parent
        H256::default(), //inter_parent
        vec![(H256::default(), config.shard_id)], //global_parents
        prop_tx_tree.root(), //prop_root
        H256::default(), //avai_root
        H256::default(), //cmt_root
        SystemTime::now(), //timestamp
    );
    
    let prop_block_2 = ProposerBlock::new(
        prop_block_header.clone(), //header
        0, //nonce
        prop_tx_tree, //prop_tx_set
    );

    multichain.insert_block_with_parent(
        VersaBlock::PropBlock(prop_block_2.clone()), //block
        &VersaHash::PropHash(prop_genesis_block.hash()), //parent
        config.shard_id, //shard_id
    ).expect("Insertion fails");

    //test get_unreferred_cmt again
    let unreferred_cmts = multichain.get_unreferred_cmt(&prop_block_2.hash());
    assert_eq!(unreferred_cmts.len(), 7);

    //compare prop_tx_set and unreferred_cmts
    let prop_tx_set_compare: HashSet<TransactionBlock> = prop_tx_set.iter().cloned().collect();
    let unreferred_cmts_compare: HashSet<TransactionBlock> = unreferred_cmts.iter().cloned().collect();

    assert!(prop_tx_set_compare == unreferred_cmts_compare);

    //insert an availability block
    let avai_tx_set: Vec<TransactionBlock> = prop_tx_set.iter().take(3).cloned().collect();
    let avai_tx_tree: MerkleTree<TransactionBlock> = MerkleTree::<TransactionBlock>::new(&avai_tx_set);
    //create a block body
    let _avai_block_content = BlockContent::create(
        avai_tx_tree.clone(),
        MerkleTree::<TransactionBlock>::new(&(vec![])),
        MerkleTree::<Transaction>::new(&(vec![]))
    );

    // let avai_genesis_block_current_shard = avai_chains.get(config.shard_id).unwrap().tip();
    let avai_block_header = BlockHeader::create(
        config.shard_id, //shard_id
        H256::default(), //prop_parent
        avai_genesis_block_current_shard.hash(), //inter_parent
        vec![(avai_genesis_block_current_shard.hash(), config.shard_id)], //global_parents
        H256::default(), //prop_root
        avai_tx_tree.root(), //avai_root
        H256::default(), //cmt_root
        SystemTime::now(), //timestamp
    );

    let avai_block_2 = AvailabilityBlock::new(
        avai_block_header,
        0,
        avai_tx_tree,
    );

    multichain.insert_block_with_parent(
        VersaBlock::ExAvaiBlock(avai_block_2.clone()), //block
        &VersaHash::ExHash(avai_genesis_block_current_shard.hash()), //parent
        config.shard_id,
    ).expect("Insertion fails");

    //test get_unreferred_cmt again
    let unreferred_cmts_2 = multichain.get_unreferred_cmt(&prop_block_2.hash());
    assert_eq!(unreferred_cmts_2.len(), 4);

    let rest_prop_tx_set: Vec<TransactionBlock> = prop_tx_set.iter().rev().take(4).cloned().collect();

    //compare rest_prop_tx_set and unreferred_cmts_2
    let rest_prop_tx_set_compare: HashSet<TransactionBlock> = rest_prop_tx_set.iter().cloned().collect();
    let unreferred_cmts_2_compare: HashSet<TransactionBlock> = unreferred_cmts_2.iter().cloned().collect();

    assert!(rest_prop_tx_set_compare == unreferred_cmts_2_compare);

    //insert an availability in other shards
    let avai_tx_tree_2: MerkleTree<TransactionBlock> = MerkleTree::<TransactionBlock>::new(&rest_prop_tx_set);
    //create a block body
    let _avai_block_content_2 = BlockContent::create(
        avai_tx_tree_2.clone(),
        MerkleTree::<TransactionBlock>::new(&(vec![])),
        MerkleTree::<Transaction>::new(&(vec![]))
    );

    let avai_block_header_2 = BlockHeader::create(
        other_shard_id, //shard_id
        H256::default(), //prop_parent
        avai_genesis_block_other_shard.hash(), //inter_parent
        vec![(avai_genesis_block_other_shard.hash(), other_shard_id)], //global_parents
        H256::default(), //prop_root
        avai_tx_tree_2.root(), //avai_root
        H256::default(), //cmt_root
        SystemTime::now(), //timestamp
    );
    let avai_block_3 = AvailabilityBlock::new(
        avai_block_header_2,
        0,
        avai_tx_tree_2,
    );

    multichain.insert_block_with_parent(
        VersaBlock::ExAvaiBlock(avai_block_3.clone()), //block
        &VersaHash::ExHash(avai_genesis_block_other_shard.hash()), //parent
        other_shard_id,
    ).expect("Insertion fails");

    //test get_unreferred_cmt again
    let unreferred_cmts_3 = multichain.get_unreferred_cmt(&prop_block_2.hash());
    assert_eq!(unreferred_cmts_3.len(), 4);

}