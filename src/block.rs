use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fmt, time::SystemTime};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub index: u64,
    pub timestamp: u64,
    pub previous_hash: String,
    pub hash: String,
    pub data: String,
    pub nonce: u64,
}

impl Block {
    pub fn new(index: u64, previous_hash: String, data: String) -> Self {
        let mut block = Self {
            index,
            timestamp: chrono::Utc::now().timestamp() as u64,
            previous_hash,
            hash: String::new(),
            data,
            nonce: 0,
        };
        block.update_hash();
        block
    }

    pub fn update_hash(&mut self) {
        let data = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.previous_hash, self.data, self.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(data);
        self.hash = hex::encode(hasher.finalize());
    }

    pub fn mine_block(&mut self, difficalty: usize) {
        let now = SystemTime::now();
        loop {
            self.update_hash();
            if self.hash[0..difficalty] == "0".repeat(difficalty) {
                break;
            }
            self.nonce += 1;
        }
        println!(
            "Block mined: index {}, hash {} for {:?}",
            self.index,
            self.hash,
            now.elapsed().unwrap(),
        );
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Block [index: {}, timestamp: {}, previous_hash: {}, hash: {}, data: {}, nonce: {}]",
            self.index, self.timestamp, self.previous_hash, self.hash, self.data, self.nonce
        )
    }
}
