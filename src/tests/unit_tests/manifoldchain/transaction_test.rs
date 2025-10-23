// #[cfg(test)]
// use rand::{self};
#[cfg(test)]
use crate::{
    manifoldchain::{
        transaction::*,
    },
    types::{
        key_pair,
        hash::{H256},
    }
};
#[cfg(test)]
// use ring::signature::{self, Ed25519KeyPair, Signature, KeyPair};
use ring::signature::{Ed25519KeyPair};


#[test]
#[allow(unused_variables)]
fn transaction_test_one() {
    let user1: H256 = (&[1u8; 32]).into();
    let user2: H256 = (&[2u8; 32]).into();
    let user3: H256 = (&[3u8; 32]).into();
    let user4: H256 = (&[4u8; 32]).into();

    let key1: Ed25519KeyPair = key_pair::random();
    let key2: Ed25519KeyPair = key_pair::random();
    let key3: Ed25519KeyPair = key_pair::random();
    let key4: Ed25519KeyPair = key_pair::random();

    let ini_tx_1 = Transaction::create_initial_tx((&user1, &key1), 10);
    let ini_tx_2 = Transaction::create_initial_tx((&user2, &key2), 10);   
    let ini_tx_3 = Transaction::create_initial_tx((&user3, &key3), 10);
    let ini_tx_4 = Transaction::create_initial_tx((&user4, &key4), 10);

    //test case
    //Initial utxo: ini_tx_1: ->1 10 
    //              ini_tx_2: ->2 10
    //              ini_tx_3: ->3 10
    //              ini_tx_4: ->4 10
    //
    //first transfer: 
    //tx1: 1->2 5, tx2: 3->4 5
    //      ->1 5,       ->3 5
    //current utxo: 1: (tx1, 1, 5 coins) 
    //              2: (ini_tx_2, 0, 10 coins) (tx1, 0, 5 coins) 
    //              3: (tx2, 1, 5 coins) 
    //              4: (ini_tx_4, 0, 10 coins) (tx2, 0, 5 coins)
    //
    let tx1 = Transaction::consume(
        vec![(&ini_tx_1, 0)],
        vec![(&user1, &key1)],
        vec![(&user2, &key2, 5), (&user1, &key1, 5)],
        TxFlag::Domestic,
    ).unwrap();
    let tx2 = Transaction::consume(
        vec![(&ini_tx_3, 0)],
        vec![(&user3, &key3)],
        vec![(&user4, &key4, 5), (&user3, &key3, 5)],
        TxFlag::Domestic,
    ).unwrap();

    assert!(Transaction::verify_owner(&tx1, vec![&ini_tx_1]));
    assert!(Transaction::verify_owner(&tx2, vec![&ini_tx_3]));

    assert_eq!(tx1.outputs.len(), 2);
    assert_eq!(tx2.outputs.len(), 2);
    //second transfer: 
    //tx3: 2(5)->1 6, tx4: 2(2)->3 4
    //     4(5)->2 2       4(2)-> 
    //         ->4 2
    //current utxo: 1: (tx1, 1, 5 coins) (tx3, 0, 6 coins) 
    //              2: (ini_tx_2, 0, 10 coins) 
    //              3: (tx2, 1, 5 coins) (tx4, 0, 4 coins) 
    //              4: (ini_tx_4, 0, 10 coins)
    //
   let tx3 = Transaction::consume(
        vec![(&tx1, 0), (&tx2, 0)],
        vec![(&user2, &key2), (&user4, &key4)],
        vec![(&user1, &key1, 6), (&user2, &key2, 2), (&user4, &key4, 2)],
        TxFlag::Domestic,
    ).unwrap();
   let tx4 = Transaction::consume(
        vec![(&tx3, 1), (&tx3, 2)],
        vec![(&user2, &key2), (&user4, &key4)],
        vec![(&user3, &key3, 4)],
        TxFlag::Domestic,
    ).unwrap();

    assert!(Transaction::verify_owner(&tx3, vec![&tx1, &tx2]));
    assert!(Transaction::verify_owner(&tx4, vec![&tx3, &tx3]));

    assert_eq!(tx3.outputs.len(), 3);
    assert_eq!(tx4.outputs.len(), 1);
}
