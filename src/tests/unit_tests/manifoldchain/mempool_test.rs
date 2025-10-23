#[cfg(test)]
use crate::{
    manifoldchain::{
        transaction::*,
        // testimony::*,
        mempool::*,
    },
    types::hash::{
        H256,
        Hashable,
    },
};
#[cfg(test)]
use std::collections::HashMap;

#[test]
fn mempool_test_one() {
    let mut mempool = Mempool::new();
    let tx1 = Transaction::gen_rand_tx();
    let tx1_hash = tx1.hash();
    assert!(mempool.insert_tx(tx1));
    assert_eq!(mempool.get_size(), 1);
    assert!(mempool.check(&tx1_hash));
    let check_tx1 = mempool.get_tx(&tx1_hash).unwrap();
    assert!(check_tx1.hash() == tx1_hash);
    match mempool.pop_one_tx() {
        (Some(tx), None) => {
            assert!(tx.hash() == tx1_hash);
        }
        _ => {
            panic!("handle one tx test failure");
        }
    }
    assert_eq!(mempool.get_size(), 0);
    assert_eq!(mempool.check(&tx1_hash), false);
    match mempool.get_tx(&tx1_hash) {
        Some(_) => {
            panic!("handle one tx test failure ");
        }
        None => {}
    }
} 

#[test]
fn mempool_test_two() {
    let mut mempool = Mempool::new();
    let tx1 = Transaction::gen_rand_tx();
    let tx1_hash = tx1.hash();
    let tx2 = Transaction::gen_rand_tx();
    let tx2_hash = tx2.hash();
    let tx3 = Transaction::gen_rand_tx();
    let tx3_hash = tx3.hash();

    assert!(mempool.insert_tx(tx1));
    assert!(mempool.insert_tx(tx2));
    assert!(mempool.insert_tx(tx3));

    assert_eq!(mempool.get_size(), 3);

    assert!(mempool.check(&tx1_hash));
    let check_tx1 = mempool.get_tx(&tx1_hash).unwrap();
    assert_eq!(check_tx1.hash(), tx1_hash);

    assert!(mempool.check(&tx2_hash));
    let check_tx2 = mempool.get_tx(&tx2_hash).unwrap();
    assert_eq!(check_tx2.hash(), tx2_hash);

    assert!(mempool.check(&tx3_hash));
    let check_tx3 = mempool.get_tx(&tx3_hash).unwrap();
    assert_eq!(check_tx3.hash(), tx3_hash);
    
    match mempool.pop_one_tx() {
        (Some(tx), None) => {
            assert_eq!(tx.hash(), tx1_hash);
        }
        _ => {
            panic!("handle multiple tx test failure");
        }
    }
    match mempool.pop_one_tx() {
        (Some(tx), None) => {
            assert_eq!(tx.hash(), tx2_hash);
        }
        _ => {
            panic!("Handle multiple tx test failure");
        }
    }
    match mempool.pop_one_tx() {
        (Some(tx), None) => {
            assert_eq!(tx.hash(), tx3_hash);
        }
        _ => {
            panic!("Handle multiple tx test failure");
        }
    }

    assert_eq!(mempool.get_size(), 0);
    assert_eq!(mempool.check(&tx1_hash), false);
    assert_eq!(mempool.check(&tx2_hash), false);
    assert_eq!(mempool.check(&tx3_hash), false);
    match mempool.get_tx(&tx1_hash) {
        Some(_) => {
            panic!("Handle multiple tx test failure");
        }
        None => {}
    }
    match mempool.get_tx(&tx2_hash) {
        Some(_) => {
            panic!("Handle multiple tx test failure");
        }
        None => {}
    }
    match mempool.get_tx(&tx3_hash) {
        Some(_) => {
            panic!("Handle multiple tx test failure");
        }
        None => {}
    }
}

#[test]
fn mempool_test_three() {
    let mut mempool = Mempool::new();

    let tx1 = Transaction::gen_rand_tx();
    let tx2 = Transaction::gen_rand_tx();
    let tx3 = Transaction::gen_rand_tx();
    let tx1_hash = tx1.hash();
    let tx2_hash = tx2.hash();
    let tx3_hash = tx3.hash();

    assert!(mempool.insert_tx(tx1));
    assert!(mempool.insert_tx(tx2));
    assert!(mempool.insert_tx(tx3));

    let txs = mempool.get_all_txs();
    let mut map: HashMap<H256, bool> = HashMap::new();
    for tx in txs.iter() {
        map.insert(tx.hash(), true);
    }
    match map.get(&tx1_hash) {
        Some(_) => {}
        None => {
            panic!("mempool_test_3 failure");
        }
    }
    match map.get(&tx2_hash) {
        Some(_) => {}
        None => {
            panic!("mempool_test_3 failure");
        }
    }
    match map.get(&tx3_hash) {
        Some(_) => {}
        None => {
            panic!("mempool_test_3 failure");
        }
    }

    let txs_hash = mempool.get_all_tx_hash();
    let mut map2: HashMap<H256, bool> = HashMap::new();

    for hash in txs_hash {
        map2.insert(hash, true);
    }

    match map2.get(&tx1_hash) {
        Some(_) => {}
        None => {
            panic!("mempool_test_3 failure");
        }
    }
    match map2.get(&tx2_hash) {
        Some(_) => {}
        None => {
            panic!("mempool_test_3 failure");
        }
    }
    match map2.get(&tx3_hash) {
        Some(_) => {}
        None => {
            panic!("mempool_test_3 failure");
        }
    }

    let deleted_txs: Vec<H256> = vec![tx2_hash.clone(), tx3_hash.clone()];
    assert!(mempool.delete_txs(deleted_txs));

    assert_eq!(mempool.check(&tx2_hash), false);
    assert_eq!(mempool.check(&tx3_hash), false);

    match mempool.get_tx(&tx2_hash) {
        Some(_) => {
            panic!("mempool_test_3 failure");
        }
        None => {}
    }
    match mempool.get_tx(&tx3_hash) {
        Some(_) => {
            panic!("mempool_test_3 failure");
        }
        None => {}
    }      

    let txs = mempool.get_all_txs();
    let txs_hash = mempool.get_all_tx_hash();
    assert_eq!(txs.len(), 1);
    assert_eq!(txs_hash.len(), 1);

    match mempool.pop_one_tx() {
        (Some(tx), None) => {
            assert_eq!(tx.hash(), tx1_hash);
        }
        _ => {
            panic!("mempool_test_3 failure");
        }
    }

    match mempool.pop_one_tx() {
        (None, None) => {}
        _ => {
            panic!("mempool_test_3 failure");
        }
    }
}

//fn mempool_test_4() {
//    let mut mempool = Mempool::new();
//
//    let tx1 = Transaction::gen_rand_tx();
//    let tx2 = Transaction::gen_rand_tx();
//    let tx3 = Transaction::gen_rand_tx();
//
//    let tx1_hash = tx1.hash();
//    let tx2_hash = tx2.hash();
//    let tx3_hash = tx3.hash();
//
//    let tmy1 = Testimony::gen_rand_tmy_by_tx(&tx1_hash);
//    let tmy2 = Testimony::gen_rand_tmy_by_tx(&tx2_hash);
//
//    let tmy1_hash = tmy1.hash();
//    let tmy2_hash = tmy2.hash();
//
//    let tmy4 = Testimony::gen_rand_tmy();
//    let tmy4_hash = tmy4.hash();
//
//    assert!(mempool.insert_tx(tx1));
//    assert!(mempool.insert_tx(tx2));
//    assert!(mempool.insert_tx(tx3));
//
//    match mempool.add_testimony(tmy1) {
//        Some(tx_hash) => {
//            assert_eq!(tx_hash, tx1_hash);
//        }
//        None => {
//            panic!("mempool_test_4 failure");
//        }
//    }
//
//    match mempool.add_testimony(tmy2) {
//        Some(tx_hash) => {
//            assert_eq!(tx_hash, tx2_hash);
//        }
//        None => {
//            panic!("mempool_test_4 failure");
//        }
//    }
//
//    match mempool.add_testimony(tmy4) {
//        None => {}
//        Some(_) => {
//            panic!("mempool_test_4 failure");
//        }
//    }
//
//    let tmy1 = mempool.get_testimony(&tmy1_hash).unwrap();
//    let tmy1_ = mempool.get_testimony_by_tx(&tx1_hash).unwrap();
//    assert_eq!(tmy1.hash(), tmy1_.hash());
//
//    let tmy2 = mempool.get_testimony(&tmy2_hash).unwrap();
//    let tmy2_ = mempool.get_testimony_by_tx(&tx2_hash).unwrap();
//    assert_eq!(tmy2.hash(), tmy2_.hash());
//
//    match mempool.get_testimony_by_tx(&tx3_hash) {
//        Some(_) => {
//            panic!("mempool_test_4 failure");
//        }
//        None => {}
//    }
//
//    assert!(mempool.remove_testimony(&tmy1_hash));
//    match mempool.get_testimony_by_tx(&tx1_hash) {
//        Some(_) => {
//            panic!("mempool_test_4 failure");
//        }
//        None => {}
//    }
//    match mempool.get_testimony(&tmy1_hash) {
//        Some(_) => {
//            panic!("mempool_test_4 failure");
//        }
//        None => {}
//    }
//
//}
//

#[test]
fn mempool_test_four() {
    let mut mempool = Mempool::new();
    let mut tx1 = Transaction::gen_rand_tx();
    let mut tx2 = Transaction::gen_rand_tx();
    let mut tx3 = Transaction::gen_rand_tx();
    let mut tx4 = Transaction::gen_rand_tx();
    let mut tx5 = Transaction::gen_rand_tx();

    tx1.flag = TxFlag::Initial;
    tx2.flag = TxFlag::Domestic;
    tx3.flag = TxFlag::Initial;
    tx4.flag = TxFlag::Domestic;
    tx5.flag = TxFlag::Initial;

    assert!(mempool.insert_tx(tx1.clone()));
    assert!(mempool.insert_tx(tx2.clone()));
    assert!(mempool.insert_tx(tx3.clone()));
    assert!(mempool.insert_tx(tx4.clone()));
    assert!(mempool.insert_tx(tx5.clone()));

    assert_eq!(mempool.get_size(), 5);

    match mempool.pop_one_tx() {
        (Some(tx), None) => {
            assert_eq!(tx.hash(), tx2.hash());
        }
        _ => {
            panic!("mempool err");
        }
    }
    match mempool.pop_one_tx() {
        (Some(tx), None) => {
            assert_eq!(tx.hash(), tx4.hash());
        }
        _ => {
            panic!("mempool err");
        }
    }
    match mempool.pop_one_tx() {
        (Some(tx), None) => {
            assert_eq!(tx.hash(), tx1.hash());
        }
        _ => {
            panic!("mempool err");
        }
    }
    match mempool.pop_one_tx() {
        (Some(tx), None) => {
            assert_eq!(tx.hash(), tx3.hash());
        }
        _ => {
            panic!("mempool err");
        }
    }
    match mempool.pop_one_tx() {
        (Some(tx), None) => {
            assert_eq!(tx.hash(), tx5.hash());
        }
        _ => {
            panic!("mempool err");
        }
    }

}

