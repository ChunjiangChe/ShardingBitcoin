use crate::{
    types::{
        merkle::MerkleTree,
        random::Random,
        hash::Hashable,
    },
    optchain::transaction::Transaction,
};

#[test]
fn test_merkle_tree() {
    let txs: Vec<Transaction> = vec![
        Transaction::random(), 
        Transaction::random(), 
        Transaction::random(),
        Transaction::random(), 
        Transaction::random(),
    ];

    let merkle_tree = MerkleTree::<Transaction>::new(&txs);
    let index: usize = 2;
    let merkle_proof = merkle_tree.proof(index);
    let data_hash = txs.get(index).unwrap().hash();
    assert!(MerkleTree::<Transaction>::verify(&merkle_tree.root(), &data_hash, &merkle_proof, index, txs.len()));
    assert!(merkle_tree.merkle_prove(&data_hash, &merkle_proof, index));

}