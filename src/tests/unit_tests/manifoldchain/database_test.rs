#[cfg(test)]
use crate::{
    manifoldchain::{
        block::{
            versa_block::*,
            exclusive_block::*,
            inclusive_block::*,
        },
        transaction::*,
        testimony::*,
    },
    types::{
        hash::{
            H256,
            Hashable,
        },
        key_pair,
    }
};
#[cfg(test)]
// use ring::signature::{self, Ed25519KeyPair, Signature, KeyPair};
use ring::signature::{Ed25519KeyPair};
#[cfg(test)]
use rocksdb::{DB, Options};
#[cfg(test)]
// use serde::{Serialize, Deserialize};

#[cfg(test)]
#[allow(unused_variables)]
fn generate_random_datatypes() 
-> (ExclusiveFullBlock, ExclusiveBlock, InclusiveFullBlock, InclusiveBlock, Transaction, Testimony) 
{
    //generate some users
    let user2: H256 = (&[2u8; 32]).into();
    let user3: H256 = (&[3u8; 32]).into();
    let user4: H256 = (&[4u8; 32]).into();
   
    let key2: Ed25519KeyPair = key_pair::random();
    let key3: Ed25519KeyPair = key_pair::random();
    let key4: Ed25519KeyPair = key_pair::random();
    //generate and insert block 1
    let tx1 = Transaction::create_initial_tx((&user2, &key2), 10);
    let tx2 = Transaction::create_initial_tx((&user4, &key4), 10);
    let tx = Transaction::consume(
        vec![(&tx1, 0)],
        vec![(&user2, &key2)],
        vec![(&user4, &key4, 5), (&user2, &key2, 5)],
        TxFlag::Domestic,
    ).unwrap();
    let tmy_units = TestimonyUnit::create(
        H256::default(),
        H256::default(),
        vec![H256::default(), H256::default(), H256::default()],
        0
    );

    let tmy = Testimony::create(
        H256::default(),
        vec![tmy_units.clone(), tmy_units.clone(), tmy_units.clone()],
    );

    let tx_size = 2048 as usize;
    let mut txs: Vec<Transaction> = vec![];
    let mut tmys: Vec<Testimony> = vec![];
    for i in 0..tx_size {
        txs.push(tx.clone());
        tmys.push(tmy.clone());
    }

    let ex_full_block = ExclusiveFullBlock::generate(
        H256::default(),
        0,
        0,
        H256::default(),
        txs.clone(),
        tmys.clone(),
        vec![H256::default(), H256::default(), H256::default()],
        vec![
            (vec![H256::default(), H256::default(), H256::default()], 0),
            (vec![H256::default(), H256::default(), H256::default()], 0),
            (vec![H256::default(), H256::default(), H256::default()], 0),
        ],
    );
    let ex_block = ex_full_block.get_exclusive_block();
    let in_full_block = InclusiveFullBlock::generate(
        H256::default(),
        0,
        0,
        H256::default(),
        txs.clone(),
        tmys.clone(),
        vec![H256::default(), H256::default(), H256::default()],
        vec![
            (vec![H256::default(), H256::default(), H256::default()], 0),
            (vec![H256::default(), H256::default(), H256::default()], 0),
            (vec![H256::default(), H256::default(), H256::default()], 0),
        ],
    );
    let in_block = in_full_block.get_inclusive_block();

//    let one_tx_size = tx.get_mem_size() as f64 / 1024.0;   
//    //let txs_vec_size = std::mem::size_of_val(&txs) as f64 / 1024.0;
//    //let tmys_vec_size = std::mem::size_of_val(&tmys) as f64 / 1024.0;
//    let ex_full_size = ex_full_block.get_mem_size() as f64 / 1024.0;
//    let ex_size = ex_block.get_mem_size() as f64 / 1024.0;
//    let in_full_size = in_full_block.get_mem_size() as f64 / 1024.0;
//    let in_size = in_block.get_mem_size() as f64 / 1024.0;
//    
//    println!("size of tx: {:.2} KB", one_tx_size);
//    //println!("size of {} txs: {:.2} KB", tx_size, txs_vec_size);
//    //println!("size of {} tmys: {:.2} KB", tx_size, tmys_vec_size);
//    println!("size of ex_full_block with {} txs: {:.2} KB", tx_size, ex_full_size);
//    println!("size of in_full_block with {} txs: {:.2} KB", tx_size, in_full_size);
//    println!("size of ex_block with {} txs: {:.2} KB", tx_size, ex_size);
//    println!("size of in_block with {} txs: {:.2} KB", tx_size, in_size);
    (ex_full_block, ex_block, in_full_block, in_block, tx, tmy)
} 

#[test]
// #[allow(unused_variables)]
fn database_test_one() {
    let path = "./DB/test_data";
    let mut options = Options::default();
    options.create_if_missing(true);
    let db = DB::open(&options, path).unwrap();

    let (ex_full_block, ex_block, in_full_block, in_block, tx, tmy) 
        = generate_random_datatypes();

    //verify exclusive_full_block
    let serialized_ex_full_block = bincode::serialize(&ex_full_block).unwrap();
    let ex_full_hash = ex_full_block.hash();
    let serialized_ex_full_hash = bincode::serialize(&ex_full_hash).unwrap();
    
    db.put(&serialized_ex_full_hash, &serialized_ex_full_block).unwrap();

    match db.get(&serialized_ex_full_hash) {
        Ok(Some(value)) => {
            let deserialized_ex_full_block: ExclusiveFullBlock = bincode::deserialize(&value).unwrap();
            assert_eq!(ex_full_hash, deserialized_ex_full_block.hash());
            let deserialized_ex_block = ex_full_block.get_exclusive_block();
            assert_eq!(deserialized_ex_block.hash(), ex_block.hash());
        }
        Ok(None) => {
            panic!("Key not found");
        }
        Err(_) => {
            panic!("Erros");
        }
    }


    //verify exclusive_block
    let serialized_ex_block = bincode::serialize(&ex_block).unwrap();
    let ex_hash = ex_block.hash();
    let serialized_ex_hash = bincode::serialize(&ex_hash).unwrap();
    
    db.put(&serialized_ex_hash, &serialized_ex_block).unwrap();

    match db.get(&serialized_ex_full_hash) {
        Ok(Some(value)) => {
            let deserialized_ex_block: ExclusiveBlock = bincode::deserialize(&value).unwrap();
            assert_eq!(ex_hash, deserialized_ex_block.hash());
        }
        Ok(None) => {
            panic!("Key not found");
        }
        Err(_) => {
            panic!("Erros");
        }
    }



    //verify inclusive_full_block
    let serialized_in_full_block = bincode::serialize(&in_full_block).unwrap();
    let in_full_hash = in_full_block.hash();
    let serialized_in_full_hash = bincode::serialize(&in_full_hash).unwrap();
    
    db.put(&serialized_in_full_hash, &serialized_in_full_block).unwrap();

    match db.get(&serialized_in_full_hash) {
        Ok(Some(value)) => {
            let deserialized_in_full_block: InclusiveFullBlock = bincode::deserialize(&value).unwrap();
            assert_eq!(in_full_hash, deserialized_in_full_block.hash());
            let deserialized_in_block = in_full_block.get_inclusive_block();
            assert_eq!(deserialized_in_block.hash(), in_block.hash());
        }
        Ok(None) => {
            panic!("Key not found");
        }
        Err(_) => {
            panic!("Erros");
        }
    }


    //verify inclusive_block
    let serialized_in_block = bincode::serialize(&in_block).unwrap();
    let in_hash = in_block.hash();
    let serialized_in_hash = bincode::serialize(&in_hash).unwrap();
    
    db.put(&serialized_in_hash, &serialized_in_block).unwrap();

    match db.get(&serialized_in_full_hash) {
        Ok(Some(value)) => {
            let deserialized_in_block: InclusiveBlock = bincode::deserialize(&value).unwrap();
            assert_eq!(in_hash, deserialized_in_block.hash());
        }
        Ok(None) => {
            panic!("Key not found");
        }
        Err(_) => {
            panic!("Erros");
        }
    }

    //verify transaction
    let serialized_tx = bincode::serialize(&tx).unwrap();
    let tx_hash = tx.hash();
    let serialized_tx_hash = bincode::serialize(&tx_hash).unwrap();
    
    db.put(&serialized_tx_hash, &serialized_tx).unwrap();

    match db.get(&serialized_tx_hash) {
        Ok(Some(value)) => {
            let deserialized_tx: Transaction = bincode::deserialize(&value).unwrap();
            assert_eq!(tx_hash, deserialized_tx.hash());
        }
        Ok(None) => {
            panic!("Key not found");
        }
        Err(_) => {
            panic!("Erros");
        }
    }

    //verify testimony
    let serialized_tmy = bincode::serialize(&tmy).unwrap();
    let tmy_hash = tmy.hash();
    let serialized_tmy_hash = bincode::serialize(&tmy_hash).unwrap();
    
    db.put(&serialized_tmy_hash, &serialized_tmy).unwrap();

    match db.get(&serialized_tmy_hash) {
        Ok(Some(value)) => {
            let deserialized_tmy: Testimony = bincode::deserialize(&value).unwrap();
            assert_eq!(tmy_hash, deserialized_tmy.hash());
        }
        Ok(None) => {
            panic!("Key not found");
        }
        Err(_) => {
            panic!("Erros");
        }
    }

    let _ = DB::destroy(&options, path);
}
