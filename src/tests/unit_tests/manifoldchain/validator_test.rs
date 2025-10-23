use crate::{
    manifoldchain::{
        multichain::*,
        blockchain::*,
        transaction::*,
        mempool::*,
        configuration::*,
        block::{
            Content,
            exclusive_block::*,
            inclusive_block::*,
            consensus_block::*,
            transaction_block::*,
            versa_block::*,
        },
        validator::*,
        testimony::*,
    },
    types::{
        hash::*,
        key_pair,
    },
};
use std::{net, process, thread, time, sync::{Arc, Mutex}};
use ring::signature::{self, Ed25519KeyPair, Signature, KeyPair};
use log::{debug, warn, error};
#[test]

fn validator_test_one() {
    let mut config0 = Configuration::new();
    config0.shard_id = 0;
    config0.shard_num = 2;
    let mut config1 = Configuration::new();
    config1.shard_id = 1;
    config1.shard_num = 2;

    let mut chain0_for_shard0 = Arc::new(Mutex::new(Blockchain::new(&config0, 0)));
    let mut chain1_for_shard0 = Arc::new(Mutex::new(Blockchain::new(&config0, 1)));
    let mut chain0_for_shard1 = Arc::new(Mutex::new(Blockchain::new(&config1, 0)));
    let mut chain1_for_shard1 = Arc::new(Mutex::new(Blockchain::new(&config1, 1)));
    
    assert_eq!(
        chain0_for_shard0.lock().unwrap().get_longest_verified_fork(), 
        chain0_for_shard1.lock().unwrap().get_longest_verified_fork()
    );
    assert_eq!(
        chain1_for_shard0.lock().unwrap().get_longest_verified_fork(),
        chain1_for_shard1.lock().unwrap().get_longest_verified_fork()
    );
    
    let mut multichain0 = Multichain::create(
        vec![&chain0_for_shard0, &chain1_for_shard0],
        &config0,
    );
    let mut multichain1 = Multichain::create(
        vec![&chain0_for_shard1, &chain1_for_shard1],
        &config1,
    );

    //generate a mempool
    let mempool0 = Arc::new(Mutex::new(Mempool::new()));
    let validator0 = Validator::new(
        &multichain0,
        &mempool0,
        &config0,
    );
    let mempool1 = Arc::new(Mutex::new(Mempool::new()));
    let validator1 = Validator::new(
        &multichain1,
        &mempool1,
        &config1,
    );


    //demo:
    //user_id: 2 and 4 in shard_0, 3 in shard_1
    //|----------|   |-----------|                 |---------|  
    //|ExFull 1  |   |ExFull 2   |     3     4     |InFull 5 |   
    //|----------|   |-----------|   |--|   |--|   |---------|   
    //|Initial-tx|<--|Domestic-tx|<--|EX|<--|In|<--|Output-tx|
    //|->2 10 (1)|   |2->4 5  (3)|   |--|   |--|   |3->2 3(5)|   
    //|->4 10 (2)|   | ->2 5     |        |------> | ->4 3   |   
    //|----------|   |4->2 5  (4)|        |        | ->3 4   |   
    //               | ->4 5     |        |  |-----|---------|   
    //               |-----------|        |  |        
    //                                    |  |      
    //                          |---------|  |      
    //                          |            |      
    //                          |            |
    //                          |            |     
    //                          |            |                        
    //                          |           \|/                        
    //|----------|   |----------|          |----------|                       
    //|ExFull 7  |   |ExFull 8  |    5     |ExFull 9  |                        
    //|----------|   |----------|   |--|   |----------|                       
    //|Initial-tx|<--|Input-tx  |<--|In|<--|Accept-tx |
    //|->3 10 (9)|   |3->2 3(10)|   |--|   |3->2 3(10)|
    //|----------|   | ->4 3    |          | ->4 3    |
    //               | ->3 4    |          | ->3 4    |
    //               |----------|          |----------|          
    //
    //
    //generate some users
    let user2: H256 = (&[2u8; 32]).into();
    let user3: H256 = (&[3u8; 32]).into();
    let user4: H256 = (&[4u8; 32]).into();
   
    let key2: Ed25519KeyPair = key_pair::random();
    let key3: Ed25519KeyPair = key_pair::random();
    let key4: Ed25519KeyPair = key_pair::random();

    //first step
    //|----------|   
    //|ExFull 1  |   
    //|----------|   
    //|Initial-tx|
    //|->2 10 (1)|  
    //|->4 10 (2)|  
    //|----------|
 
    //generate and insert block 1
    let tx1 = Transaction::create_initial_tx((&user2, &key2), 10);
    let tx2 = Transaction::create_initial_tx((&user4, &key4), 10);
    
    let genesis_hash0 = multichain0.get_longest_verified_fork();
    let genesis_hash1 = multichain1.get_longest_verified_fork();
    let block1 = ExclusiveFullBlock::generate(
        genesis_hash0.clone(),
        config0.shard_id,
        0,
        config0.difficulty.clone(),
        vec![tx1.clone(), tx2.clone()],
        vec![],
        vec![genesis_hash0.clone()],
        vec![(vec![genesis_hash0.clone()], config0.shard_id)],
    );
    let blk1 = block1.get_exclusive_block();

    match multichain0.insert_block_with_parent(
        VersaBlock::ExFullBlock(block1.clone()),
        &genesis_hash0,
        config0.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    match multichain1.insert_block_with_parent(
        VersaBlock::ExBlock(blk1.clone()),
        &genesis_hash0,
        config0.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
     
    //second step: generate block 2 and check its validity
    //|----------|   |-----------|
    //|ExFull 1  |   |ExFull 2   |
    //|----------|   |-----------|
    //|Initial-tx|<--|Domestic-tx|
    //|->2 10 (1)|   |2->4 5  (3)|
    //|->4 10 (2)|   | ->2 5     |
    //|----------|   |4->2 5  (4)|      
    //               | ->4 5     |        
    //               |-----------|        
    //                              

    let tx3 = Transaction::consume(
        vec![(&tx1, 0)],
        vec![(&user2, &key2)],
        vec![(&user4, &key4, 5), (&user2, &key2, 5)],
        TxFlag::Domestic,
    ).unwrap();
    let tx4 = Transaction::consume(
        vec![(&tx2, 0)],
        vec![(&user4, &key4)],
        vec![(&user2, &key2, 5), (&user4, &key4, 5)],
        TxFlag::Domestic,
    ).unwrap();
    let block2 = ExclusiveFullBlock::generate(
        block1.hash(),
        config0.shard_id,
        0,
        config0.difficulty.clone(),
        vec![tx3.clone(), tx4.clone()],
        vec![],
        vec![(block1.hash())],
        vec![(vec![block1.hash()], config0.shard_id)],
    );
    let blk2 = block2.get_exclusive_block();
    
    let versa_block2 = VersaBlock::ExFullBlock(block2.clone());
    match validator0.validate_block(&versa_block2) {
        Ok(_) => {}
        Err(_) => {
            panic!("Validation error");
        }
    }
    match validator1.validate_block(&VersaBlock::ExBlock(blk2.clone())) {
        Ok(_) => {}
        Err(_) => {
            panic!("Validation error");
        }
    }   
    match multichain0.insert_block_with_parent(
        versa_block2.clone(),
        &block1.hash(),
        config0.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }

    match multichain1.insert_block_with_parent(
        VersaBlock::ExBlock(blk2.clone()),
        &block1.hash(),
        config0.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }

    //third step
    //|----------|   |-----------|              
    //|ExFull 1  |   |ExFull 2   |    3
    //|----------|   |-----------|   |--|   
    //|Initial-tx|<--|Domestic-tx|<--|EX|
    //|->2 10 (1)|   |2->4 5  (3)|   |--|   
    //|->4 10 (2)|   | ->2 5     |        
    //|----------|   |4->2 5  (4)|        
    //               | ->4 5     |        
    //               |-----------|      
    
    //generate the third block
    let (blk3, _) = ExclusiveBlock::generate(
        block2.hash(),
        config0.shard_id,
        0,
        config0.difficulty.clone(),
        vec![],
        vec![],
        vec![(block2.hash())],
        vec![(vec![block2.hash()], config0.shard_id)]
    );
    match validator0.validate_block(&VersaBlock::ExBlock(blk3.clone())) {
        Ok(_) => {}
        Err(_) => {
            panic!("Validation error");
        }
    }
    match validator1.validate_block(&VersaBlock::ExBlock(blk3.clone())) {
        Ok(_) => {}
        Err(_) => {
            panic!("Validation error");
        }
    }

    match multichain0.insert_block_with_parent(
        VersaBlock::ExBlock(blk3.clone()),
        &block2.hash(),
        config0.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    match multichain1.insert_block_with_parent(
        VersaBlock::ExBlock(blk3.clone()),
        &block2.hash(),
        config0.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }

    //|----------|   |-----------|                 
    //|ExFull 1  |   |ExFull 2   |     3     4     
    //|----------|   |-----------|   |--|   |--|   
    //|Initial-tx|<--|Domestic-tx|<--|EX|<--|In|
    //|->2 10 (1)|   |2->4 5  (3)|   |--|   |--|   
    //|->4 10 (2)|   | ->2 5     |        
    //|----------|   |4->2 5  (4)|        
    //               | ->4 5     |        
    //               |-----------|        
 
    let (blk4, _) = InclusiveBlock::generate(
        block2.hash(),
        config0.shard_id,
        0,
        config0.difficulty.clone(),
        vec![],
        vec![],
        vec![(blk3.hash())],
        vec![(vec![blk3.hash()], config0.shard_id)]
    );
    match validator0.validate_block(&VersaBlock::InBlock(blk4.clone())) {
        Ok(_) => {}
        Err(_) => {
            panic!("Validation error");
        }
    }
    match validator1.validate_block(&VersaBlock::InBlock(blk4.clone())) {
        Ok(_) => {}
        Err(_) => {
            panic!("Validation error");
        }
    }
    match multichain0.insert_block_with_parent(
        VersaBlock::InBlock(blk4.clone()),
        &blk3.hash(),
        config0.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    match multichain1.insert_block_with_parent(
        VersaBlock::InBlock(blk4.clone()),
        &blk3.hash(),
        config0.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    //|----------|   |-----------|                 
    //|ExFull 1  |   |ExFull 2   |     3     4     
    //|----------|   |-----------|   |--|   |--|   
    //|Initial-tx|<--|Domestic-tx|<--|EX|<--|In|
    //|->2 10 (1)|   |2->4 5  (3)|   |--|   |--|   
    //|->4 10 (2)|   | ->2 5     |        
    //|----------|   |4->2 5  (4)|      
    //               | ->4 5     |        
    //               |-----------|      
    //                                    
    //                          
    //                  
    //                          
    //                      
    //                          
    //                      
    //|----------|   
    //|ExFull 7  |   
    //|----------|   
    //|Initial-tx|
    //|->3 10 (9)|   
    //|----------|   
  
    let tx9 = Transaction::create_initial_tx((&user3, &key3), 10);
    let block7 = ExclusiveFullBlock::generate(
        genesis_hash1.clone(),
        config1.shard_id,
        0,
        config1.difficulty.clone(),
        vec![tx9.clone()],
        vec![],
        vec![genesis_hash1.clone()],
        vec![(vec![genesis_hash1.clone()], config1.shard_id)],
    );
    let blk7 = block7.get_exclusive_block();
    match validator1.validate_block(&VersaBlock::ExFullBlock(block7.clone())) {
        Ok(_) => {}
        Err(_) => {
            panic!("Validation error");
        }
    }
    match validator0.validate_block(&VersaBlock::ExBlock(blk7.clone())) {
        Ok(_) => {}
        Err(_) => {
            panic!("Validation error");
        }
    }


    match multichain1.insert_block_with_parent(
        VersaBlock::ExFullBlock(block7.clone()),
        &genesis_hash1,
        config1.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    match multichain0.insert_block_with_parent(
        VersaBlock::ExBlock(blk7.clone()),
        &genesis_hash1,
        config1.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    //user_id: 2 and 4 in shard_0, 3 in shard_1
    //|----------|   |-----------|                 
    //|ExFull 1  |   |ExFull 2   |     3     4     
    //|----------|   |-----------|   |--|   |--|   
    //|Initial-tx|<--|Domestic-tx|<--|EX|<--|In|
    //|->2 10 (1)|   |2->4 5  (3)|   |--|   |--|   
    //|->4 10 (2)|   | ->2 5     |        
    //|----------|   |4->2 5  (4)|        
    //               | ->4 5     |        
    //               |-----------|      
    //                                  
    //                          
    //                  
    //                      
    //                      
    //                          
    //                      
    //|----------|   |----------|                            
    //|ExFull 7  |   |ExFull 8  |            
    //|----------|   |----------|   
    //|Initial-tx|<--|Input-tx  |
    //|->3 10 (9)|   |3->2 3(10)|   
    //|----------|   | ->4 3    |   
    //               | ->3 4    |          
    //               |----------|          
    //                                     
    
    let tx10 = Transaction::consume(
        vec![(&tx9, 0)],
        vec![(&user3, &key3)],
        vec![(&user2, &key2, 3), (&user4, &key4, 3), (&user3, &key3, 4)],
        TxFlag::Input,
    ).unwrap();
    let block8 = ExclusiveFullBlock::generate(
        block7.hash(),
        config1.shard_id,
        0,
        config1.difficulty.clone(),
        vec![tx10.clone()],
        vec![],
        vec![block7.hash()],
        vec![(vec![block7.hash()], config1.shard_id)],
    );
    let blk8 = block8.get_exclusive_block();
    let versa_block8 = VersaBlock::ExFullBlock(block8.clone());
    
    let tmy10 = Testimony::generate(
        &tx10,
        &versa_block8,
        0,
        config1.shard_id,
        config1.shard_num,
        true,
    ).unwrap();

    match multichain1.insert_block_with_parent(
        versa_block8.clone(),
        &block7.hash(),
        config1.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    match multichain0.insert_block_with_parent(
        VersaBlock::ExBlock(blk8.clone()),
        &block7.hash(),
        config1.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    //|----------|   |-----------|                 |---------|  
    //|ExFull 1  |   |ExFull 2   |     3     4     |InFull 5 |   
    //|----------|   |-----------|   |--|   |--|   |---------|   
    //|Initial-tx|<--|Domestic-tx|<--|EX|<--|In|<--|Output-tx|
    //|->2 10 (1)|   |2->4 5  (3)|   |--|   |--|   |3->2 3(5)|   
    //|->4 10 (2)|   | ->2 5     |        |------> | ->4 3   |   
    //|----------|   |4->2 5  (4)|        |        | ->3 4   |   
    //               | ->4 5     |        |        |---------|   
    //               |-----------|        |          
    //                                    |        
    //                          |---------|        
    //                          |                  
    //                          |            
    //                          |                 
    //                          |                                    
    //                          |                                   
    //|----------|   |----------|          
    //|ExFull 7  |   |ExFull 8  |    5     
    //|----------|   |----------|   |--|                          
    //|Initial-tx|<--|Input-tx  |<--|In|
    //|->3 10 (9)|   |3->2 3(10)|   |--|   
    //|----------|   | ->4 3    |          
    //               | ->3 4    |          
    //               |----------|          
    //
 
    let mut tx5 = tx10.clone();
    tx5.flag = TxFlag::Output;
    
    //generate block 5
    let block5 = InclusiveFullBlock::generate(
        block2.hash(),
        config0.shard_id,
        0,
        config0.difficulty.clone(),
        vec![tx5.clone()],
        vec![tmy10.clone()],
        vec![blk4.hash()],
        vec![(vec![blk4.hash()], 0), (vec![block8.hash()], 1)]
    );

    let blk5 = block5.get_inclusive_block();
    
    match validator1.validate_block(&VersaBlock::InBlock(blk5.clone())) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block validation");
        }
    }
    
    match multichain1.insert_block_with_parent(
        VersaBlock::InBlock(blk5.clone()),
        &block8.hash(),
        config1.shard_id
    ){
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    match multichain0.insert_block_with_parent(
        VersaBlock::InBlock(blk5.clone()),
        &block8.hash(),
        config1.shard_id
    ){
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }

    match validator0.validate_block(&VersaBlock::InFullBlock(block5.clone())) {
        Ok(_) => {}
        Err(e) => {
            println!("{:?}", e);
            panic!("Error in block validation");
        }
    }  
    match multichain1.insert_block_with_parent(
        VersaBlock::InBlock(blk5.clone()),
        &blk4.hash(),
        config0.shard_id
    ){
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    match multichain0.insert_block_with_parent(
        VersaBlock::InFullBlock(block5.clone()),
        &blk4.hash(),
        config0.shard_id
    ){
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    //|----------|   |-----------|                 |---------|  
    //|ExFull 1  |   |ExFull 2   |     3     4     |InFull 5 |   
    //|----------|   |-----------|   |--|   |--|   |---------|   
    //|Initial-tx|<--|Domestic-tx|<--|EX|<--|In|<--|Output-tx|
    //|->2 10 (1)|   |2->4 5  (3)|   |--|   |--|   |3->2 3(5)|   
    //|->4 10 (2)|   | ->2 5     |        |------> | ->4 3   |   
    //|----------|   |4->2 5  (4)|        |        | ->3 4   |   
    //               | ->4 5     |        |        |---------|   
    //               |-----------|        |                 |        
    //                                    |                 |      
    //                          |---------|                 |      
    //                          |                           |      
    //                          |                           |
    //                          |                           |     
    //                          |                           |                        
    //                          |                          \|/                        
    //|----------|   |----------|          |----------|   |----------|                       
    //|ExFull 7  |   |ExFull 8  |    5     |ExFull 6  |   |ExFull 9  |                        
    //|----------|   |----------|   |--|   |----------|   |----------|                       
    //|Initial-tx|<--|Input-tx  |<--|In|<--|Output-tx |<--|Accept-tx |
    //|->3 10 (9)|   |3->2 3(10)|   |--|   |3->2 3(12)|   |3->2 3(11)|
    //|----------|   | ->4 3    |          | ->4 3    |   | ->4 3    |
    //               | ->3 4    |          | ->3 4    |   | ->3 4    |
    //               |----------|          |----------|   |----------|       
    //
    //
    
    let mut tx12 = tx5.clone();
    
    let block6 = ExclusiveFullBlock::generate(
        block8.hash(),
        config1.shard_id,
        0,
        config1.difficulty.clone(),
        vec![tx12.clone()],
        vec![tmy10.clone()],
        vec![blk5.hash()],
        vec![(vec![blk5.hash()], config1.shard_id)]
    );
    let blk6 = block6.get_exclusive_block();
    match validator1.validate_block(&VersaBlock::ExFullBlock(block6.clone())){
        Ok(_) => {}
        Err(proof) => {
            println!("{:?}", proof);
            panic!("Error in validation");
        }
    }
    match multichain1.insert_block_with_parent(
        VersaBlock::ExFullBlock(block6.clone()),
        &blk5.hash(),
        config1.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    match multichain0.insert_block_with_parent(
        VersaBlock::ExBlock(blk6.clone()),
        &blk5.hash(),
        config1.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }   
    let tmy6 = Testimony::generate(
        &tx12,
        &VersaBlock::ExFullBlock(block6),
        0,
        config1.shard_id,
        config1.shard_num,
        true,
    ).unwrap();

    
    let mut tx11 = tx5.clone();
    tx11.flag = TxFlag::Accept;
    let tmy5 = Testimony::generate(
        &tx5,
        &VersaBlock::InFullBlock(block5),
        0,
        config0.shard_id,
        config0.shard_num,
        true,
    ).unwrap();
    let _ = tmy5.get_ori_blk_hash(tx11.outputs[0].hash()).unwrap();
    let mut tmy_units: Vec<TestimonyUnit> = vec![];
    tmy_units.extend(tmy6.get_tmy_units());
    tmy_units.extend(tmy5.get_tmy_units());
    let tmy_5_6 = Testimony::create(
        tx11.hash(),
        tmy_units
    );

    let block9 = ExclusiveFullBlock::generate(
        block8.hash(),
        config1.shard_id,
        0,
        config1.difficulty.clone(),
        vec![tx11.clone()],
        vec![tmy_5_6.clone()],
        vec![blk5.hash()],
        vec![(vec![blk5.hash()], config1.shard_id)]
    );
    let tx11_2 = &block9.get_txs_ref()[0];
    assert_eq!(tx11_2.hash(), tx11.hash());
    assert_eq!(tx11_2.outputs[0].hash(), tx11.outputs[0].hash());
    let blk9 = block9.get_exclusive_block();
    match validator1.validate_block(&VersaBlock::ExFullBlock(block9.clone())){
        Ok(_) => {}
        Err(proof) => {
            println!("{:?}", proof);
            panic!("Error in validation");
        }
    }
    match multichain1.insert_block_with_parent(
        VersaBlock::ExFullBlock(block9.clone()),
        &blk5.hash(),
        config1.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }
    match multichain0.insert_block_with_parent(
        VersaBlock::ExBlock(blk9.clone()),
        &blk5.hash(),
        config1.shard_id
    ) {
        Ok(_) => {}
        Err(_) => {
            panic!("Error in block insertion");
        }
    }

}
