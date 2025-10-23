use crate::{
    types::{
        hash::{
            H256,
            Hashable,
        },
    }
};
// use rocksdb::{DB, Options, DBIterator, Direction, IteratorMode};
use rocksdb::{DB, Options, IteratorMode};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
};

pub struct Database<T>
    where T: Hashable + Default + Serialize + DeserializeOwned,
{
    pub path: String,
    pub db: DB,
    pub sample_data: T,
    counter: usize,
}

impl<T> Database<T>
    where T: Hashable + Default + Serialize + DeserializeOwned,
{
    pub fn new(path: String) -> Self {
        let mut options = Options::default();
        options.create_if_missing(true);
        let absolute_path = format!("./DB/{}", path);
        let db = DB::open(&options, absolute_path.clone()).unwrap();

        Self {
            path: absolute_path,
            db,
            sample_data: T::default(),
            counter: 0,
        }
    }

    pub fn insert(&mut self, hash: H256, data: T) -> Result<bool, String> {
        let serialized_key = bincode::serialize(&hash).unwrap();
        let serialized_value = bincode::serialize(&data).unwrap();

        match self.db.put(&serialized_key, &serialized_value) {
            Ok(_) => {
                self.counter += 1;
                Ok(true)
            }
            Err(_) => Err(String::from("Insertion fails")),
        }
    }

    pub fn get(&self, hash: &H256) -> Option<T> {
        let serialized_key = bincode::serialize(hash).unwrap();

        match self.db.get(&serialized_key) {
            Ok(Some(data)) => {
                let deserialized_data: T = bincode::deserialize(&data).unwrap();
                Some(deserialized_data)
            }
            _ => None,
        }
    }

    pub fn contains_key(&self, hash: &H256) -> bool {
        let serialized_key = bincode::serialize(hash).unwrap();

        match self.db.get(&serialized_key) {
            Ok(Some(_)) => true,
            _ => false, 
        }
    }

    pub fn iter(&self) -> std::vec::IntoIter<(H256, T)> {
        let mut all_data: Vec<(H256, T)> = vec![];
        for item in self.db.iterator(IteratorMode::Start) {
            match item {
                Ok((key, value)) => {
                    let deserialized_key: H256 = bincode::deserialize(&key).unwrap();
                    let deserialized_value: T = bincode::deserialize(&value).unwrap();
                    all_data.push((deserialized_key, deserialized_value));
                }
                Err(_) => {}
            }
        }
        all_data.into_iter()
    }
    pub fn remove(&mut self, hash: &H256) {
        if self.contains_key(hash) {
            let serialized_key = bincode::serialize(hash).unwrap();
            self.db.delete(&serialized_key).unwrap();
            self.counter -= 1;
        }
    }

    pub fn len(&self) -> usize {
        self.counter
    }

    pub fn into_map(&self) -> HashMap<H256, T> {
        let mut all_data: Vec<(H256, T)> = vec![];
        for item in self.db.iterator(IteratorMode::Start) {
            match item {
                Ok((key, value)) => {
                    let deserialized_key: H256 = bincode::deserialize(&key).unwrap();
                    let deserialized_value: T = bincode::deserialize(&value).unwrap();
                    all_data.push((deserialized_key, deserialized_value));
                }
                Err(_) => {}
            }
        }
        all_data.into_iter().collect()    
    }

   // pub fn destroy(&self) {
   //     let _ = DB::destroy(&self.options, &self.path);
   // }
}
