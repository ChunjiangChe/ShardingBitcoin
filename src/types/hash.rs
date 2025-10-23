use serde::{Serialize, Deserialize};
use std::convert::TryInto;
// #[cfg(any(test, test_utilities))]
use rand::Rng;
use array_init::array_init;
use hex::{encode, decode};
use crate::{
    types::{
        random::Random,
    },
};

/// An object that can be meaningfully hashed.
pub trait Hashable {
    /// Hash the object using SHA256.
    fn hash(&self) -> H256;
    fn chash(hash1: &H256, hash2: &H256) -> H256 {
        let hash12_array: [u8; 64] = array_init(|i|
            if i < 32 {
                hash1.0[i]
            } else {
                hash2.0[i-32]
            }
        as u8);
        ring::digest::digest(&ring::digest::SHA256, &hash12_array).into()
    }
    // fn multi_hash(hashs: &Vec<H256>) -> H256 {
    //     let size: usize = hashs.len();
    //     let values: Vec<u8> = (0..size*32).map( |i| {
    //         let index: usize = i / 32;
    //         let offset: usize = i % 32;
    //         hashs[index].0[offset]
    //     }).collect();
    //     ring::digest::digest(&ring::digest::SHA256, values.as_slice()).into()
    // }
    fn multi_hash(hashes: &Vec<H256>) -> H256 {
        let s: usize = hashes.len();
        if s == 0 {
            return H256::default();
        } else if s == 1 {
            return hashes[0].clone();
        } else if s == 2 {
            return Self::chash(&hashes[0], &hashes[1]);
        } else {
            let middle = (s+1)/2;
            let left_slice = (&hashes[0..middle]).to_vec();
            let right_slice = (&hashes[middle..s]).to_vec();
            return Self::chash(&(Self::multi_hash(&left_slice)), &(Self::multi_hash(&right_slice)));
        }
    }

    fn pow_hash(base: &H256, nonce: u32) -> H256 {
        let bytes: [u8; 4] = [
            ((nonce >> 24) & 0xFF) as u8,
            ((nonce >> 16) & 0xFF) as u8,
            ((nonce >> 8) & 0xFF) as u8,
            (nonce & 0xFF) as u8,
        ];
        let mut array: [u8; 36] = [0; 36];
        array[0..32].copy_from_slice(&base.0);
        array[32..36].copy_from_slice(&bytes);
        ring::digest::digest(&ring::digest::SHA256, &array).into()
    }
}

/// A SHA256 hash.
#[derive(Eq, PartialEq, Serialize, Deserialize, Clone, Hash, Copy)]
pub struct H256(pub [u8; 32]); // big endian u256

impl Default for H256 {
    fn default() -> Self {
        (&[255u8; 32]).into()
    }
}

impl Random for H256 {
    fn random() -> Self {
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        let mut raw_bytes = [0; 32];
        raw_bytes.copy_from_slice(&random_bytes);
        (&raw_bytes).into()
    }
}

impl H256 {
    pub fn get_mem_size() -> usize {
        std::mem::size_of::<u8>() * 32
    }
}



impl Hashable for H256 {
    fn hash(&self) -> H256 {
        ring::digest::digest(&ring::digest::SHA256, &self.0).into()
    }
    
}

impl<T: Hashable> Hashable for Vec<T> {
    fn hash(&self) -> H256 {
        let hash_vec: Vec<H256> = self.iter()
            .map(|x| x.hash())
            .collect();
        <T as Hashable>::multi_hash(&hash_vec)
    }
}

impl std::fmt::Display for H256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let start = if let Some(precision) = f.precision() {
            if precision >= 64 {
                0
            } else {
                32 - precision / 2
            }
        } else {
            0
        };
        for byte_idx in start..32 {
            write!(f, "{:>02x}", &self.0[byte_idx])?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for H256 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{:>02x}{:>02x}..{:>02x}{:>02x}",
            &self.0[0], &self.0[1], &self.0[30], &self.0[31]
        )
    }
}

impl std::convert::AsRef<[u8]> for H256 {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::convert::From<&[u8; 32]> for H256 {
    fn from(input: &[u8; 32]) -> H256 {
        let mut buffer: [u8; 32] = [0; 32];
        buffer[..].copy_from_slice(input);
        H256(buffer)
    }
}

impl std::convert::From<&H256> for [u8; 32] {
    fn from(input: &H256) -> [u8; 32] {
        let mut buffer: [u8; 32] = [0; 32];
        buffer[..].copy_from_slice(&input.0);
        buffer
    }
}

impl std::convert::From<[u8; 32]> for H256 {
    fn from(input: [u8; 32]) -> H256 {
        H256(input)
    }
}

impl std::convert::From<H256> for [u8; 32] {
    fn from(input: H256) -> [u8; 32] {
        input.0
    }
}


impl std::convert::From<H256> for String {
    fn from(input: H256) -> String {
        let bytes: [u8; 32] = input.into();
        encode(&bytes) 
    }
}

impl std::convert::From<String> for H256 {
    fn from(input: String) -> H256 {
        match decode(&input) {
            Ok(bytes) => {
                if bytes.len() == 32 {
                    let mut result = [0u8; 32];
                    result.copy_from_slice(&bytes);
                    H256(result)
                } else {
                    panic!("Invalid length when converting String to H256");
                }
            }
            Err(_) => {
                panic!("Failed to convert String to H256");
            }
        }
    }
}



impl std::convert::From<ring::digest::Digest> for H256 {
    fn from(input: ring::digest::Digest) -> H256 {
        let mut raw_hash: [u8; 32] = [0; 32];
        raw_hash[0..32].copy_from_slice(input.as_ref());
        H256(raw_hash)
    }
}

impl Ord for H256 {
    fn cmp(&self, other: &H256) -> std::cmp::Ordering {
        let self_higher = u128::from_be_bytes(self.0[0..16].try_into().unwrap());
        let self_lower = u128::from_be_bytes(self.0[16..32].try_into().unwrap());
        let other_higher = u128::from_be_bytes(other.0[0..16].try_into().unwrap());
        let other_lower = u128::from_be_bytes(other.0[16..32].try_into().unwrap());
        let higher = self_higher.cmp(&other_higher);
        match higher {
            std::cmp::Ordering::Equal => self_lower.cmp(&other_lower),
            _ => higher,
        }
    }
}

impl PartialOrd for H256 {
    fn partial_cmp(&self, other: &H256) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// #[cfg(any(test, test_utilities))]
// #[cfg(any(test))]
// pub fn generate_random_hash() -> H256 {
//     let mut rng = rand::thread_rng();
//     let random_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
//     let mut raw_bytes = [0; 32];
//     raw_bytes.copy_from_slice(&random_bytes);
//     (&raw_bytes).into()
// }
