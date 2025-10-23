use super::hash::{Hashable, H256};
use serde::{Serialize, Deserialize};

/// A Merkle tree.
#[derive(Serialize, Deserialize, Debug, Default, Clone, Eq, Hash, PartialEq)]
pub struct MerkleTree<T> 
where  T: Hashable + Clone,
{
    pub root: H256,
    pub data: Vec<T>,
}



impl<T> MerkleTree<T> 
where T: Hashable + Clone,
{
    pub fn new(data: &[T]) -> Self {
        let size: usize = data.len();
        if size <= 0 {
            let data_vec: Vec<T> = Vec::new();
            let root_bytes: [u8; 32] = [0; 32];
            let root: H256 = H256::from(&root_bytes);
            MerkleTree {
                root,
                data: data_vec
            }
        } else {
            let data_vec: Vec<T>  = (0..size).map(|i| data[i].clone()).collect();
            let fixed_vec: Vec<H256> = (0..size).map(|i| data_vec[i].hash()).collect();
            let root: H256 = Self::recursive_hash(&fixed_vec, (0, fixed_vec.len()));
            MerkleTree {
                root,
                data: data_vec,
            }
        }
    }

    pub fn root(&self) -> H256 {
        self.root.clone()
    }

    /// Returns the Merkle Proof of data at index i
    pub fn proof(&self, index: usize) -> Vec<H256> {
        let size: usize = self.data.len();
        assert!(index < size, "index ({}) must be smaller than leaf size ({})", index, size);
        let hash_vec: Vec<H256> = (0..size).map(|i| self.data[i].hash()).collect();
        Self::recursive_proof(&hash_vec, index, (0, hash_vec.len()))
    }


    fn recursive_verify(
        proof: &[H256], 
        index: usize, 
        data_range: (usize, usize), 
        proof_range: (usize, usize)
    ) -> H256 {
        let (data_start, data_end): (usize, usize) = data_range;
        let (proof_start, proof_end): (usize, usize) = proof_range;
        let size: usize = data_end - data_start;
        assert!(size > 0);
        if data_start == data_end - 1 {
            proof[proof_start].clone()
        } else if data_start == data_end - 2 {
            H256::chash(&proof[proof_start], &proof[proof_end-1])
        } else {
            let mid: usize = data_start + size/2;
            if index < mid {
                let hash1: H256 = Self::recursive_verify(
                    proof, 
                    index, 
                    (data_start, mid), 
                    (proof_start, proof_end - 1)
                );
                let hash2: H256 = proof[proof_end-1].clone();
                H256::chash(&hash1, &hash2)
            } else {
                let hash1: H256 = proof[proof_start].clone();
                let hash2: H256 = Self::recursive_verify(
                    proof, 
                    index, 
                    (mid, data_end), 
                    (proof_start + 1, proof_end)
                );
                H256::chash(&hash1, &hash2)
            }
        }
    }

    fn get_proof_index(index: usize, range: (usize, usize)) -> usize {
        let (start, end): (usize, usize) = range;
        let size: usize = end - start;
        assert!(size > 0); 
        if start == end - 1 {
            0
        } else {
            let mid: usize = start + size/2;
            if index < mid {
                Self::get_proof_index(index, (start, mid))
            } else {
                Self::get_proof_index(index, (mid, end)) + 1
            }
        }
    }
    fn recursive_hash(leaves: &Vec<H256>, range: (usize, usize)) -> H256 {
        let (start, end): (usize, usize) = range;
        let size: usize = end - start;
        assert!(size > 0);
        if start == end - 1 {
            leaves[start].clone()
        } else if start == end - 2 {
            H256::chash(&leaves[start], &leaves[end-1])
        } else {
            let hash1: H256 = Self::recursive_hash(leaves, (start, start + size/2));
            let hash2: H256 = Self::recursive_hash(leaves, (start + size/2, end));
            H256::chash(&hash1, &hash2)
        }
    }

    fn recursive_proof(data: &Vec<H256>, index: usize, range: (usize, usize)) -> Vec<H256> {
        let (start, end): (usize, usize) = range;
        let size: usize = end - start;
        assert!(size > 0);
        let mut res: Vec<H256> = Vec::new();
        if index >= start && index < end {
            if start == end - 1 {
                res.push(data[start].clone());
            } else if start == end - 2 {
                res.push(data[start].clone());
                res.push(data[start+1].clone());
            } else {
                let vec1: Vec<H256> = Self::recursive_proof(
                    data, 
                    index, 
                    (start, start + size/2)
                );
                let vec2: Vec<H256> = Self::recursive_proof(
                    data, 
                    index, 
                    (start + size/2, end)
                );
                for item in vec1 {
                    res.push(item);
                }
                for item in vec2 {
                    res.push(item);
                }
            } 
        } else {
                let curr_hash: H256 = Self::recursive_hash(data, (start, end));
                res.push(curr_hash);
        }
        res
    }

    /// Verify that the datum hash with a vector of proofs will produce the Merkle root. 
    /// Also need the index of datum and `leaf_size`, the total number of leaves.
    pub fn verify(
        root: &H256, 
        datum: &H256, 
        proof: &[H256], 
        index: usize, 
        leaf_size: usize) -> bool 
    {
        assert!(index < leaf_size);
        let generated_hash: H256 = Self::recursive_verify(proof, index, (0, leaf_size), (0, proof.len()));
        let con1: bool = generated_hash == *root;
        let proof_index: usize = Self::get_proof_index(index, (0, leaf_size));
        let con2: bool =  proof[proof_index] == *datum;
        con1 && con2
    }

    pub fn merkle_prove(&self, 
        datum: &H256, 
        proof: &[H256], 
        index: usize) -> bool 
    {
        let leaf_size: usize = self.data.len();   
        assert!(index < leaf_size);
        let generated_hash: H256 = Self::recursive_verify(proof, index, (0, leaf_size), (0, proof.len()));
        let con1: bool = generated_hash == self.root;
        let proof_index: usize = Self::get_proof_index(index, (0, leaf_size));
        let con2: bool =  proof[proof_index] == *datum;
        con1 && con2
    }
}
