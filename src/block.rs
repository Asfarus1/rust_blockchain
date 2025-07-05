use crate::errors::{Error, Result};
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
        block.hash = block.compute_hash();
        block
    }

    pub fn compute_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}",
            self.index, self.timestamp, self.previous_hash, self.data, self.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(data);
        hex::encode(hasher.finalize())
    }

    pub fn mine_block(&mut self, difficulty: usize) -> Result<()> {
        let now = SystemTime::now();
        loop {
            self.hash = self.compute_hash();
            if self.hash[0..difficulty] == "0".repeat(difficulty) {
                println!(
                    "Successfully mined block {} with hash {} in {:.2?}",
                    self.index,
                    self.hash,
                    now.elapsed()?
                );
                break;
            }
            self.nonce += 1;
        }
        Ok(())
    }

    pub fn validate(&self, previous_hash: &str, difficulty: usize) -> Result<()> {
        if self.hash != self.compute_hash() {
            Err(Error::BlockHasInvalidHash(self.index, self.hash.to_owned()))?;
        }
        if self.hash[..difficulty] != "0".repeat(difficulty) {
            Err(Error::UnsatisfiedHashDifficulty(
                self.index,
                self.hash.to_owned(),
            ))?;
        }
        if self.previous_hash != previous_hash {
            Err(Error::BlockHasInvalidPreviusBlockHash(
                self.index,
                self.hash.to_owned(),
                previous_hash.to_owned(),
            ))?;
        }
        Ok(())
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
