#[cfg(test)]
use crate::{
    types::{
        hash::H256,
    },
};


#[test]
fn hash_test_one() {
    let hash = H256::default();
    let str: String = hash.into();
    let hash_2: H256 = str.into();
    assert_eq!(hash, hash_2);
}
