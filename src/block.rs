use serde::{Deserialize, Serialize};

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
        Self {
            index,
            timestamp: chrono::Utc::now().timestamp() as u64,
            previous_hash,
            hash: String::new(),
            data,
            nonce: 0,
        }
    }
}
