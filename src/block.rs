use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

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
        block.hash = block.calculate_hash();
        block
    }

    pub fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.previous_hash, self.data, self.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
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
