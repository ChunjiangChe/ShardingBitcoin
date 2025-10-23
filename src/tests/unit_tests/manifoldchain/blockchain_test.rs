use rand::{self, Rng};
use crate::{
    manifoldchain::{
        transaction::*,
        blockchain::*,
        block::{
            exclusive_block::*,
            inclusive_block::*,
            transaction_block::*,
            versa_block::*,
        },
        testimony::*,
        configuration::*,
        validator::*,
    },
    types::hash::{H256, Hashable},
};

fn gen_rand_hash() -> H256 {
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();

    (&bytes).into()
}


#[test]
#[allow(unused_variables)]
fn node_test_one() {
    let root_hash = gen_rand_hash();
    let mut root = Box::new(Node {
        val: root_hash.clone(),
        children: Vec::new(),
        height: 0,
        longest_height: 1,
    });

    let hash1 = gen_rand_hash();
    let hash2 = gen_rand_hash();
    let hash3 = gen_rand_hash();
    let hash4 = gen_rand_hash();
    let hash5 = gen_rand_hash();
    let hash6 = gen_rand_hash();

    //First construction: 
    //root->1->2
    //       ->3
    //       ->4

    Node::insert(
        &mut root,
        &root_hash,
        hash1.clone(),
        2
    );
    Node::insert(
        &mut root,
        &hash1,
        hash2.clone(),
        2
    );
    Node::insert(
        &mut root,
        &hash1,
        hash3.clone(),
        2
    );
    Node::insert(
        &mut root,
        &hash1,
        hash4.clone(),
        2
    );


    assert_eq!(Node::get_leaves(&root).len(), 3);

    match Node::get_node_by_hash(&root, &root.val) {
        Some(node) => {
            assert_eq!(node.val, root.val);
            assert_eq!(node.children.len(), 1);
            assert_eq!(node.height, 0);
            assert_eq!(node.longest_height, 2);
        }
        None => {
            panic!("failure in checking node in the middle");
        }
    }
    
    match Node::get_node_by_hash(&root, &hash1) {
        Some(node) => {
            assert_eq!(node.val, hash1);
            assert_eq!(node.children.len(), 3);
            assert_eq!(node.height, 1);
            assert_eq!(node.longest_height, 2);
        }
        None => {
            panic!("failure in checking node in the middle");
        }
    } 
    match Node::get_node_by_hash(&root, &hash2) {
        Some(node) => {
            assert_eq!(node.val, hash2);
            assert_eq!(node.children.len(), 0);
            assert_eq!(node.height, 2);
            assert_eq!(node.longest_height, 2);
        }
        None => {
            panic!("failure in checking node in the middle");
        }
    } 
    match Node::get_node_by_hash(&root, &hash3) {
        Some(node) => {
            assert_eq!(node.val, hash3);
            assert_eq!(node.children.len(), 0);
            assert_eq!(node.height, 2);
            assert_eq!(node.longest_height, 2);
        }
        None => {
            panic!("failure in checking node in the middle");
        }
    } 
    match Node::get_node_by_hash(&root, &hash4) {
        Some(node) => {
            assert_eq!(node.val, hash4);
            assert_eq!(node.children.len(), 0);
            assert_eq!(node.height, 2);
            assert_eq!(node.longest_height, 2);
        }
        None => {
            panic!("failure in checking node in the middle");
        }
    } 
    
    //Second construction:
    //root->1->2->5
    //          ->6
    //       ->3
    //       ->4
    Node::insert(
        &mut root,
        &hash2,
        hash5.clone(),
        2
    );
    Node::insert(
        &mut root,
        &hash2,
        hash6.clone(),
        2
    );


    assert_eq!(Node::get_leaves(&root).len(), 4);


    match Node::get_node_by_hash(&root, &root_hash) {
        Some(node) => {
            assert_eq!(node.val, root_hash);
            assert_eq!(node.children.len(), 1);
            assert_eq!(node.height, 0);
            assert_eq!(node.longest_height, 3);
        }
        None => {
            panic!("failure in checking node in the middle");
        }
    } 
    match Node::get_node_by_hash(&root, &hash1) {
        Some(node) => {
            assert_eq!(node.val, hash1);
            assert_eq!(node.children.len(), 3);
            assert_eq!(node.height, 1);
            assert_eq!(node.longest_height, 3);
        }
        None => {
            panic!("failure in checking node in the middle");
        }
    } 
    match Node::get_node_by_hash(&root, &hash2) {
        Some(node) => {
            assert_eq!(node.val, hash2);
            assert_eq!(node.children.len(), 2);
            assert_eq!(node.height, 2);
            assert_eq!(node.longest_height, 3);
        }
        None => {
            panic!("failure in checking node in the middle");
        }
    } 
    match Node::get_node_by_hash(&root, &hash5) {
        Some(node) => {
            assert_eq!(node.val, hash5);
            assert_eq!(node.children.len(), 0);
            assert_eq!(node.height, 3);
            assert_eq!(node.longest_height, 3);
        }
        None => {
            panic!("failure in checking node in the middle");
        }
    } 
    match Node::get_node_by_hash(&root, &hash6) {
        Some(node) => {
            assert_eq!(node.val, hash6);
            assert_eq!(node.children.len(), 0);
            assert_eq!(node.height, 3);
            assert_eq!(node.longest_height, 3);
        }
        None => {
            panic!("failure in checking node in the middle");
        }
    } 
    //Third construction
    //root->1->2->5
    //       ->3
    //       ->4
    match Node::prune(&mut root, &hash6) {
        Some(deleted_hash) => {
            assert!(deleted_hash.contains(&hash6));
        }
        None => {
            panic!("failure in testing prune");
        }
    }

    assert_eq!(Node::get_leaves(&root).len(), 3);

    match Node::get_node_by_hash(&root, &hash6) {
        Some(_) => {
            panic!("failure in testing prunning");
        }
        None => {}
    }

    match Node::get_node_by_hash(&root, &hash2) {
        Some(node) => {
            assert_eq!(node.val, hash2);
            assert_eq!(node.children.len(), 1);
            assert_eq!(node.height, 2);
            assert_eq!(node.longest_height, 3);
        }
        None => {
            panic!("failure in testing prunning function");
        }
    }
    //Forth Construction
    //root->1->3
    //       ->4
    match Node::prune(&mut root, &hash2) {
        Some(deleted_hash) => {
            assert!(deleted_hash.contains(&hash2));
            assert!(deleted_hash.contains(&hash5));
        }
        None => {
            panic!("failure in testing prunning function");
        }
    }

    match Node::get_node_by_hash(&root, &hash5) {
        Some(_) => {
            panic!("failure in testing prunning");
        }
        None => {}
    }

    match Node::get_node_by_hash(&root, &hash2) {
        Some(_) => {
            panic!("failure in testing prunning");
        }
        None => {}
    }

    match Node::get_node_by_hash(&root, &hash1) {
        Some(node) => {
            assert_eq!(node.val, hash1);
            assert_eq!(node.children.len(), 2);
            assert_eq!(node.height, 1);
            assert_eq!(node.longest_height, 2);
        }
        None => {
            panic!("failure in testing prunning");
        }
    }

    match Node::get_node_by_hash(&root, &root_hash) {
        Some(node) => {
            assert_eq!(node.val, root_hash);
            assert_eq!(node.children.len(), 1);
            assert_eq!(node.height, 0);
            assert_eq!(node.longest_height, 2);
        }
        None => {
            panic!("failure in testing prunning");
        }
    }
    
}

#[test]
#[allow(unused_variables)]
fn blockchain_test_two() {
    //demo:
    //user_id: 2 and 4 in shard_0, 3 in shard_1
    //|----------|   |-----------|                 |---------|   |---------|
    //|ExFull    |   |ExFull     |                 |InFull   |   |ExFull   |
    //|----------|   |-----------|   |--|   |--|   |---------|   |---------|
    //|Initial-tx|<--|Domestic-tx|<--|EX|<--|In|<--|Output-tx|<--|Accept-tx|
    //|->2 10    |   |2->4 5     |   |--|   |--|   |3->2 3   |   |2->3 1   |
    //|->4 10    |   | ->2 5     |                 | ->4 3   |   | ->2 4   |
    //|----------|   |4->2 5     |                 | ->3 4   |   |4->3 1   |
    //               | ->4 5     |                 |---------|   | ->4 4   |
    //               |-----------|                 |Input-tx |   |---------|
    //                                             |2->3 1   |
    //                                             | ->2 4   |
    //                                             |4->3 1   |
    //                                             | ->4 4   |
    //                                             |---------|
    //
    //generate some users
    let user2: H256 = (&[2u8; 32]).into();
    let user3: H256 = (&[3u8; 32]).into();
    let user4: H256 = (&[4u8; 32]).into();

    //generate initial transaction
    let uin2 = UtxoInput::default();
    let uout2 = UtxoOutput {
        receiver_addr: user2.clone(),
        value: 10,
        public_key_ref: Vec::new(),
    };
    let inputs_2 = vec![uin2];
    let outputs_2 = vec![uout2];
    let ini_tx_2 = Transaction {
        inputs: inputs_2,
        outputs: outputs_2,
        flag: TxFlag::Initial,
    };

//        let uin3 = UtxoInput::default();
//        let uout3 = UtxoOutput {
//            receiver_addr: user3.clone(),
//            value: 10,
//            public_key_ref: Vec::new(),
//        };
//        let inputs_3 = vec![uin3];
//        let outputs_3 = vec![uout3];
//        let ini_tx_3 = Transaction {
//            inputs: inputs_3,
//            outputs: outputs_3,
//            flag: TxFlag::Initial,
//        };
    
    let uin4 = UtxoInput::default();
    let uout4 = UtxoOutput {
        receiver_addr: user4.clone(),
        value: 10,
        public_key_ref: Vec::new(),
    };
    let inputs_4 = vec![uin4];
    let outputs_4 = vec![uout4];
    let ini_tx_4 = Transaction {
        inputs: inputs_4,
        outputs: outputs_4,
        flag: TxFlag::Initial,
    };

    let mut config = Configuration::new();
    config.shard_id = 0;
    config.shard_num = 2;

    //generate the first block
    let mut blockchain = Blockchain::new(&config, config.shard_id);
    let genesis_hash = blockchain.tip();
    let ex_full_block_1 = ExclusiveFullBlock::generate(
        genesis_hash.clone(),
        config.shard_id,
        0,
        config.difficulty.clone(),
        vec![ini_tx_2.clone(), ini_tx_4.clone()],
        vec![],
        vec![genesis_hash.clone()],
        vec![(vec![genesis_hash.clone()], config.shard_id)],
    );

    match blockchain.insert_block_with_parent(
        VersaBlock::ExFullBlock(ex_full_block_1.clone()),
        &genesis_hash,
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("failure in inserting block 1");
        }
    }

    //generate the second block
    let uin_2_4_5 = UtxoInput {
        sender_addr: user2.clone(),
        tx_hash: ini_tx_2.hash(),
        value: 10,
        index: 0,
        sig_ref: vec![]
    };
    let uout_2_4_5 = UtxoOutput {
        receiver_addr: user4.clone(),
        value: 5,
        public_key_ref: vec![]
    };
    let uout_2_2_5 = UtxoOutput {
        receiver_addr: user2.clone(),
        value: 5,
        public_key_ref: vec![]
    };

    let tx_2_1 = Transaction {
        inputs: vec![uin_2_4_5],
        outputs: vec![uout_2_4_5, uout_2_2_5],
        flag: TxFlag::Domestic,
    };

    let uin_4_2_5 = UtxoInput {
        sender_addr: user4.clone(),
        tx_hash: ini_tx_4.hash(),
        value: 10,
        index: 0,
        sig_ref: vec![]
    };
    let uout_4_2_5 = UtxoOutput {
        receiver_addr: user2.clone(),
        value: 5,
        public_key_ref: vec![]
    };
    let uout_4_4_5 = UtxoOutput {
        receiver_addr: user4.clone(),
        value: 5,
        public_key_ref: vec![]
    };

    let tx_2_2 = Transaction {
        inputs: vec![uin_4_2_5],
        outputs: vec![uout_4_2_5, uout_4_4_5],
        flag: TxFlag::Domestic,
    };

   let ex_full_block_2 = ExclusiveFullBlock::generate(
        ex_full_block_1.hash(),
        config.shard_id,
        0,
        config.difficulty.clone(),
        vec![tx_2_1.clone(), tx_2_2.clone()],
        vec![],
        vec![ex_full_block_1.hash()],
        vec![(vec![ex_full_block_1.hash()], config.shard_id)],
    );
    match blockchain.insert_block_with_parent(
        VersaBlock::ExFullBlock(ex_full_block_2.clone()),
        &ex_full_block_1.hash(),
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("failure in inserting block 2");
        }
    }

    // generate the three block
    let (ex_block_3, tx_block_null) = ExclusiveBlock::generate(
        ex_full_block_2.hash(),
        config.shard_id,
        0,
        config.difficulty.clone(),
        vec![],
        vec![],
        vec![ex_full_block_2.hash()],
        vec![(vec![ex_full_block_2.hash()], config.shard_id)],
    );
    match blockchain.insert_block_with_parent(
        VersaBlock::ExBlock(ex_block_3.clone()),
        &ex_full_block_2.hash(),
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("failure in inserting block 3");
        }
    }

    // generate the fourth block
    let (in_block_4, tx_block_null) = InclusiveBlock::generate(
        ex_full_block_2.hash(),
        config.shard_id,
        0,
        config.difficulty.clone(),
        vec![],
        vec![],
        vec![ex_block_3.hash()],
        vec![(vec![ex_block_3.hash()], config.shard_id)]
    );
    match blockchain.insert_block_with_parent(
        VersaBlock::InBlock(in_block_4.clone()), 
        &ex_block_3.hash()
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("failure in inserting block 4");
        }
    }
    
    //generate the fifth inclusive full block
    //generate an output-tx
    let uin_3_2_3 = UtxoInput {
        sender_addr: user3.clone(),
        tx_hash: H256::default(),
        value: 10, 
        index: 0,
        sig_ref: vec![]
    };
    let uout_3_2_3 = UtxoOutput {
        receiver_addr: user2.clone(),
        value: 3,
        public_key_ref: vec![],
    };
    let uout_3_4_3 = UtxoOutput {
        receiver_addr: user4.clone(),
        value: 3,
        public_key_ref: vec![],
    };
    let uout_3_3_4 = UtxoOutput {
        receiver_addr: user3.clone(),
        value: 4,
        public_key_ref: vec![],
    };
    let tx_5_1 = Transaction {
        inputs: vec![uin_3_2_3],
        outputs: vec![uout_3_2_3, uout_3_4_3, uout_3_3_4],
        flag: TxFlag::Output,
    };
    let testimony_5_1 = Testimony::create(
        tx_5_1.hash(),
        vec![],
    );
    
    //generate an input-tx
    let uin_2_3_1 = UtxoInput {
        sender_addr: user2.clone(),
        tx_hash: tx_2_1.hash(),
        value: 5,
        index: 1,
        sig_ref: vec![],
    };
    let uout_2_3_1 = UtxoOutput {
        receiver_addr: user3.clone(),
        value: 1,
        public_key_ref: vec![],
    };
    let uout_2_2_4 = UtxoOutput {
        receiver_addr: user2.clone(),
        value: 4,
        public_key_ref: vec![],
    };
    let tx_5_2 = Transaction {
        inputs: vec![uin_2_3_1],
        outputs: vec![uout_2_3_1, uout_2_2_4],
        flag: TxFlag::Input,
    };
    //generate another input-tx
    let uin_4_3_1 = UtxoInput {
        sender_addr: user4.clone(),
        tx_hash: tx_2_2.hash(),
        value: 5,
        index: 1,
        sig_ref: vec![],
    };
    let uout_4_3_1 = UtxoOutput {
        receiver_addr: user3.clone(),
        value: 1,
        public_key_ref: vec![],
    };
    let uout_4_4_4 = UtxoOutput {
        receiver_addr: user4.clone(),
        value: 4,
        public_key_ref:vec![],
    };
    let tx_5_3 = Transaction {
        inputs: vec![uin_4_3_1],
        outputs: vec![uout_4_3_1, uout_4_4_4],
        flag: TxFlag::Input,
    };
    
    let in_full_block_5 = InclusiveFullBlock::generate(
        ex_full_block_2.hash(),
        config.shard_id,
        0,
        config.difficulty.clone(),
        vec![tx_5_1.clone(), tx_5_2.clone(), tx_5_3.clone()],
        vec![testimony_5_1.clone()],
        vec![in_block_4.hash()],
        vec![(vec![in_block_4.hash()], config.shard_id)],
    );


    match blockchain.insert_block_with_parent(
        VersaBlock::InFullBlock(in_full_block_5.clone()),
        &in_block_4.hash(),
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("failure in block insertion");
        }
    }

    //generate the last block
    let mut tx_6_1 = tx_5_2.clone();
    let mut tx_6_2 = tx_5_3.clone();

    tx_6_1.flag = TxFlag::Accept;
    tx_6_2.flag = TxFlag::Reject;

    let testimony_6_1 = Testimony::create(
        tx_6_1.hash(),
        vec![],
    );
    let testimony_6_2 = Testimony::create(
        tx_6_2.hash(),
        vec![],
    );

    let ex_full_block_6 = ExclusiveFullBlock::generate(
        ex_full_block_2.hash(),
        config.shard_id,
        0,
        config.difficulty.clone(),
        vec![tx_6_1.clone(), tx_6_2.clone()],
        vec![testimony_6_1.clone(), testimony_6_2.clone()],
        vec![in_full_block_5.hash()],
        vec![(vec![in_full_block_5.hash()], config.shard_id)],
    );

    match blockchain.insert_block_with_parent(
        VersaBlock::ExFullBlock(ex_full_block_6.clone()),
        &in_full_block_5.hash(),
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("failure in block insertion");
        }
    }

    //verify whether the shard_id is correctly calculated
    if config.shard_id != Validator::get_shard_id(
        &user2,
        config.shard_num,
    ) {
        panic!("Wrong shard id");
    }
    if config.shard_id != Validator::get_shard_id(
        &user4,
        config.shard_num,
    ) {
        panic!("Wrong shard id");
    }
    
    //Now let us check whether the state is correctly created
    let states = blockchain.get_states();
    
    //check the first exclusive full block's state
    let ex_full_state_1 = states.get(&ex_full_block_1.hash()).unwrap();

    let (utxo_1_1, possible_tmy) = ex_full_state_1.get(&(ini_tx_2.hash(), 0)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_1_1.hash() != ini_tx_2.hash() {
        panic!("Wrong Utxo");
    }
    let (utxo_1_2, possible_tmy) = ex_full_state_1.get(&(ini_tx_4.hash(), 0)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_1_2.hash() != ini_tx_4.hash() {
        panic!("Wrong Utxo");
    }

    //check the second exclusive full block's state
    let ex_full_state_2 = states.get(&(ex_full_block_2.hash())).unwrap();

    assert_eq!(ex_full_state_2.len(), 4);
    let (utxo_2_1, possible_tmy) = ex_full_state_2.get(&(tx_2_1.hash(), 0)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_2_1.hash() != tx_2_1.hash() {
        panic!("Wrong Utxo");
    }
    let (utxo_2_2, possible_tmy) = ex_full_state_2.get(&(tx_2_1.hash(), 1)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_2_2.hash() != tx_2_1.hash() {
        panic!("Wrong Utxo");
    }
    let (utxo_2_3, possible_tmy) = ex_full_state_2.get(&(tx_2_2.hash(), 0)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_2_3.hash() != tx_2_2.hash() {
        panic!("Wrong Utxo");
    }
    let (utxo_2_4, possible_tmy) = ex_full_state_2.get(&(tx_2_2.hash(), 1)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_2_4.hash() != tx_2_2.hash() {
        panic!("Wrong Utxo");
    }

    //check the third exclusive block
    let ex_state_3 = states.get(&(ex_block_3.hash())).unwrap();

    assert_eq!(ex_state_3.len(), 4);
    let (utxo_3_1, possible_tmy) = ex_state_3.get(&(tx_2_1.hash(), 0)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_3_1.hash() != tx_2_1.hash() {
        panic!("Wrong Utxo");
    }
    let (utxo_3_2, possible_tmy) = ex_state_3.get(&(tx_2_1.hash(), 1)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_3_2.hash() != tx_2_1.hash() {
        panic!("Wrong Utxo");
    }
    let (utxo_3_3, possible_tmy) = ex_state_3.get(&(tx_2_2.hash(), 0)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_3_3.hash() != tx_2_2.hash() {
        panic!("Wrong Utxo");
    }
    let (utxo_3_4, possible_tmy) = ex_state_3.get(&(tx_2_2.hash(), 1)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_3_4.hash() != tx_2_2.hash() {
        panic!("Wrong Utxo");
    }

    //check the forth inclusive block's state
    let in_state_4 = states.get(&(in_block_4.hash())).unwrap();

    assert_eq!(in_state_4.len(), 4);
    let (utxo_4_1, possible_tmy) = in_state_4.get(&(tx_2_1.hash(), 0)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_4_1.hash() != tx_2_1.hash() {
        panic!("Wrong Utxo");
    }
    let (utxo_4_2, possible_tmy) = in_state_4.get(&(tx_2_1.hash(), 1)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_4_2.hash() != tx_2_1.hash() {
        panic!("Wrong Utxo");
    }
    let (utxo_4_3, possible_tmy) = in_state_4.get(&(tx_2_2.hash(), 0)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_4_3.hash() != tx_2_2.hash() {
        panic!("Wrong Utxo");
    }
    let (utxo_4_4, possible_tmy) = in_state_4.get(&(tx_2_2.hash(), 1)).unwrap();
    if possible_tmy.is_some() {
        panic!("Wrong Testimony");
    }
    if utxo_4_4.hash() != tx_2_2.hash() {
        panic!("Wrong Utxo");
    }

    //check the 5th inclusive full block's state
    let in_full_state_5 = states.get(&in_full_block_5.hash()).unwrap();
    assert_eq!(in_full_state_5.len(), 4);
    if let (utxo_5_1, Some(_)) = in_full_state_5.get(&(tx_5_1.hash(), 0)).unwrap() {
        assert_eq!(utxo_5_1.hash(), tx_5_1.hash());
    } else {
        panic!("Wrong state");
    }
    if let (utxo_5_2, Some(_)) = in_full_state_5.get(&(tx_5_1.hash(), 1)).unwrap() {
        assert_eq!(utxo_5_2.hash(), tx_5_1.hash());
    } else {
        panic!("Wrong state");
    }
    if let (utxo_5_5, None) = in_full_state_5.get(&(tx_2_1.hash(), 0)).unwrap() {
        assert_eq!(utxo_5_5.hash(), tx_2_1.hash());
    } else {
        panic!("Wrong state");
    }
    if let (utxo_5_6, None) = in_full_state_5.get(&(tx_2_2.hash(), 0)).unwrap() {
        assert_eq!(utxo_5_6.hash(), tx_2_2.hash());
    } else {
        panic!("Wrong state");
    }

    //check the last exclusive full block's state
    let ex_full_state_6 = states.get(&ex_full_block_6.hash()).unwrap();
    assert_eq!(ex_full_state_6.len(), 5);
    if let (utxo_6_1, Some(_)) = ex_full_state_6.get(&(tx_6_2.hash(), 0)).unwrap() {
        assert_eq!(utxo_6_1.hash(), tx_6_2.hash());
    } else {
        panic!("Wrong state");
    }

    //validate some basic information
    assert_eq!(blockchain.longest_chain_hash, ex_full_block_6.hash());
    assert_eq!(blockchain.longest_verified_chain_hash, ex_full_block_2.hash());
    assert_eq!(blockchain.height, 6);
    assert_eq!(blockchain.verified_height, 2);
    
    if let None = blockchain.get_block(&ex_full_block_1.hash()) {
        panic!("error in getting block");
    } 
    if let None = blockchain.get_block(&ex_full_block_2.hash()) {
        panic!("error in getting block");
    }
    if let None = blockchain.get_block(&ex_block_3.hash()) {
        panic!("error in getting block");
    }       
    if let None = blockchain.get_block(&in_block_4.hash()) {
        panic!("error in getting block");
    }
    if let None = blockchain.get_block(&in_full_block_5.hash()) {
        panic!("error in getting block");
    }
    if let None = blockchain.get_block(&ex_full_block_6.hash()) {
        panic!("error in getting block");
    }      

    //test all_blocks_in_longest_chain function
    let history = blockchain.all_blocks_in_longest_chain();
    assert!(history.contains(&ex_full_block_1.hash()));
    assert!(history.contains(&ex_full_block_2.hash()));
    assert!(history.contains(&ex_block_3.hash()));
    assert!(history.contains(&in_block_4.hash()));       
    assert!(history.contains(&in_full_block_5.hash()));
    assert!(history.contains(&ex_full_block_6.hash()));

    //insert some other blocks and test other functions
    //|----------|   |-----------|                 |---------|   |---------|
    //|ExFull    |   |ExFull     |                 |InFull   |   |ExFull   |
    //|----------|   |-----------|   |--|   |--|   |---------|   |---------|
    //|Initial-tx|<--|Domestic-tx|<--|EX|<--|In|<--|Output-tx|<--|Accept-tx|
    //|->2 10    |   |2->4 5     |   |--|   |--|   |3->2 3   |   |2->3 1   |
    //|->4 10    |   | ->2 5     |                 | ->4 3   |   | ->2 4   |
    //|----------|   |4->2 5     |                 | ->3 4   |   |4->3 1   |
    //               | ->4 5     |                 |---------|   | ->4 4   |
    //               |-----------|                 |Input-tx |   |---------|
    //                                             |2->3 1   |
    //                                             | ->2 4   |
    //                                             |4->3 1   |
    //                                             | ->4 4   |
    //                                             |---------|<--|--|
    //                                                           |In|
    //                                                           |--|
    //
    //
    //generate the 7th inclusive block
    let (in_block_7, _) = InclusiveBlock::generate(
        ex_full_block_2.hash(),
        config.shard_id,
        0,
        config.difficulty.clone(),
        vec![],
        vec![],
        vec![in_full_block_5.hash()],
        vec![(vec![in_full_block_5.hash()], config.shard_id)]
    );
    match blockchain.insert_block_with_parent(
        VersaBlock::InBlock(in_block_7.clone()),
        &in_full_block_5.hash()
    ) {
        Ok(_) => {}
        Err(err) => {
            println!("{err}");
            panic!("failure in inserting block 7");
        }
    }

    let ver_sta_1 = blockchain.get_verify_status(&ex_full_block_1.hash()).unwrap();
    let ver_sta_2 = blockchain.get_verify_status(&ex_full_block_2.hash()).unwrap();
    let ver_sta_3 = blockchain.get_verify_status(&ex_block_3.hash()).unwrap();
    let ver_sta_4 = blockchain.get_verify_status(&in_block_4.hash()).unwrap();
    let ver_sta_5 = blockchain.get_verify_status(&in_full_block_5.hash()).unwrap();
    let ver_sta_6 = blockchain.get_verify_status(&ex_full_block_6.hash()).unwrap();
    let ver_sta_7 = blockchain.get_verify_status(&in_block_7.hash()).unwrap();

    assert_eq!(ver_sta_1, VerStatus::Verified);
    assert_eq!(ver_sta_2, VerStatus::Verified);
    assert_eq!(ver_sta_3, VerStatus::Unverified);
    assert_eq!(ver_sta_4, VerStatus::Unverified);
    assert_eq!(ver_sta_5, VerStatus::Verified);
    assert_eq!(ver_sta_6, VerStatus::Verified);
    assert_eq!(ver_sta_7, VerStatus::Unverified);


    //test all_blocks_end_with_block function
    let history_2 = blockchain.all_blocks_end_with_block(&in_block_7.hash()).unwrap();
    assert!(history_2.contains(&ex_full_block_1.hash()));
    assert!(history_2.contains(&ex_full_block_2.hash()));
    assert!(history_2.contains(&ex_block_3.hash()));
    assert!(history_2.contains(&in_block_4.hash()));
    assert!(history_2.contains(&in_full_block_5.hash()));
    assert!(history_2.contains(&in_block_7.hash()));

    //check whether the leaves are correct
    let leaves = blockchain.get_leaves();
    assert!(leaves.contains(&ex_full_block_6.hash()));
    assert!(leaves.contains(&in_block_7.hash()));

    //check is_block_confirmed function
    assert!(blockchain.is_block_confirmed(&ex_block_3.hash(), 3));
    assert!(!blockchain.is_block_confirmed(&in_block_4.hash(), 3));

    //check is_block_in_longest_chain function
    //insert some other blocks and test other functions
    //|----------|   |-----------|                 |---------|   |---------|
    //|ExFull    |   |ExFull     |                 |InFull   |   |ExFull   |
    //|----------|   |-----------|   |--|   |--|   |---------|   |---------|
    //|Initial-tx|<--|Domestic-tx|<--|EX|<--|In|<--|Output-tx|<--|Accept-tx|
    //|->2 10    |   |2->4 5     |   |--|   |--|   |3->2 3   |   |2->3 1   |
    //|->4 10    |   | ->2 5     |                 | ->4 3   |   | ->2 4   |
    //|----------|   |4->2 5     |                 | ->3 4   |   |4->3 1   |
    //               | ->4 5     |                 |---------|   | ->4 4   |
    //               |-----------|                 |Input-tx |   |---------|
    //                                             |2->3 1   |
    //                                             | ->2 4   |
    //                                             |4->3 1   |
    //                                             | ->4 4   |
    //                                             |---------|<--|--|<--|--|
    //                                                           |In|   |Ex|
    //                                                           |--|   |--|
    //
    //

    let (ex_block_8, _) = ExclusiveBlock::generate(
        ex_full_block_2.hash(),
        config.shard_id,
        0,
        config.difficulty.clone(),
        vec![],
        vec![],
        vec![in_block_7.hash()],
        vec![(vec![in_block_7.hash()], config.shard_id)]
    );
    match blockchain.insert_block_with_parent(
        VersaBlock::ExBlock(ex_block_8.clone()),
        &in_block_7.hash()
    ) {
        Ok(_) => {}
        Err(err) => {
            println!("{err}");
            panic!("failure in inserting block 7");
        }
    }
    assert!(blockchain.is_block_in_longest_chain(&ex_full_block_1.hash()));
    assert!(blockchain.is_block_in_longest_chain(&ex_full_block_2.hash()));
    assert!(blockchain.is_block_in_longest_chain(&ex_block_3.hash()));
    assert!(blockchain.is_block_in_longest_chain(&in_block_4.hash()));
    assert!(blockchain.is_block_in_longest_chain(&in_full_block_5.hash()));
    assert!(!blockchain.is_block_in_longest_chain(&ex_full_block_6.hash()));
    assert!(blockchain.is_block_in_longest_chain(&in_block_7.hash()));
    assert!(blockchain.is_block_in_longest_chain(&ex_block_8.hash()));

    //check prune_fork function
    //|----------|   |-----------|                 |---------|   |---------|
    //|ExFull    |   |ExFull     |                 |InFull   |   |ExFull   |
    //|----------|   |-----------|   |--|   |--|   |---------|   |---------|
    //|Initial-tx|<--|Domestic-tx|<--|EX|<--|In|<--|Output-tx|<--|Accept-tx|
    //|->2 10    |   |2->4 5     |   |--|   |--|   |3->2 3   |   |2->3 1   |
    //|->4 10    |   | ->2 5     |                 | ->4 3   |   | ->2 4   |
    //|----------|   |4->2 5     |                 | ->3 4   |   |4->3 1   |
    //               | ->4 5     |                 |---------|   | ->4 4   |
    //               |-----------|                 |Input-tx |   |---------|
    //                                             |2->3 1   |
    //                                             | ->2 4   |
    //                                             |4->3 1   |
    //                                             | ->4 4   |
    //                                             |---------|
    //                                                          
    //                                                           
    //
    //

    blockchain.prune_fork(&in_block_7.hash());
    match blockchain.get_block(&in_block_7.hash()) {
        Some(_) => {
            panic!("Prunning error");
        }
        None => {}
    }
    match blockchain.get_block(&ex_block_8.hash()) {
        Some(_) => {
            panic!("Prunning error");
        }
        None => {}
    }

    let _ = blockchain.get_block(&in_block_4.hash()).unwrap();
    let _ = blockchain.get_block(&in_full_block_5.hash()).unwrap();
    let _ = blockchain.get_block(&ex_full_block_6.hash()).unwrap();

    //validate some basic information
    assert_eq!(blockchain.longest_chain_hash, ex_full_block_6.hash());
    assert_eq!(blockchain.longest_verified_chain_hash, ex_full_block_2.hash());
    assert_eq!(blockchain.height, 6);
    assert_eq!(blockchain.verified_height, 2);

    //check whether the leaves are correct
    let leaves = blockchain.get_leaves();
    assert!(leaves.contains(&ex_full_block_6.hash()));
    assert_eq!(leaves.len(), 1);

    //check verify_block function
    blockchain.verify_block(&ex_block_3.hash());
    //validate some basic information
    assert_eq!(blockchain.longest_chain_hash, ex_full_block_6.hash());
    assert_eq!(blockchain.longest_verified_chain_hash, ex_block_3.hash());
    assert_eq!(blockchain.height, 6);
    assert_eq!(blockchain.verified_height, 3);

    //check get_block_height function
    assert_eq!(blockchain.get_block_height(&ex_full_block_1.hash()).unwrap(), 1);
    assert_eq!(blockchain.get_block_height(&ex_full_block_2.hash()).unwrap(), 2);
    assert_eq!(blockchain.get_block_height(&ex_block_3.hash()).unwrap(), 3);
    assert_eq!(blockchain.get_block_height(&in_block_4.hash()).unwrap(), 4);
    assert_eq!(blockchain.get_block_height(&in_full_block_5.hash()).unwrap(), 5);
    assert_eq!(blockchain.get_block_height(&ex_full_block_6.hash()).unwrap(), 6);
 
    

}


