use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use tracing::instrument;

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
    #[instrument(name = "create_new_block", level = "debug")]
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

    #[instrument(fields(index=self.index), skip_all, level = "debug")]
    pub fn mine_block(&mut self, difficulty: usize) -> Result<()> {
        loop {
            self.hash = self.compute_hash();
            if self.hash[0..difficulty] == "0".repeat(difficulty) {
                break;
            }
            self.nonce += 1;
        }
        Ok(())
    }

    #[instrument(level = "debug", name = "validate_block")]
    pub fn validate(&self, previous_hash: &str, difficulty: usize) -> Result<()> {
        if self.previous_hash != previous_hash {
            Err(Error::BlockHasInvalidPreviusBlockHash(
                self.index,
                self.hash.to_owned(),
                previous_hash.to_owned(),
            ))?;
        }
        if self.hash[..difficulty] != "0".repeat(difficulty) {
            Err(Error::UnsatisfiedHashDifficulty(self.index, difficulty))?;
        }
        if self.hash != self.compute_hash() {
            Err(Error::BlockHasInvalidHash(self.index, self.hash.to_owned()))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Error;

    #[test]
    fn test_new_block_initialization() {
        let block = Block::new(1, "0000".to_string(), "Test Data".to_string());
        assert_eq!(block.index, 1);
        assert_eq!(block.previous_hash, "0000");
        assert_eq!(block.data, "Test Data");
        assert!(!block.hash.is_empty());
    }

    #[test]
    fn test_compute_hash_changes_with_nonce() {
        let mut block = Block::new(1, "0000".to_string(), "Test".to_string());
        let original_hash = block.hash.clone();
        block.nonce += 1;
        let new_hash = block.compute_hash();
        assert_ne!(original_hash, new_hash);
    }

    #[test]
    fn test_mine_block_validates_difficulty() {
        let mut block = Block::new(1, "0000".to_string(), "Mining Test".to_string());
        let difficulty = 2;
        block.mine_block(difficulty).unwrap();
        assert!(block.hash.starts_with(&"0".repeat(difficulty)));
    }

    #[test]
    fn test_validate_block_success() {
        let difficulty = 2;
        let mut block = Block::new(1, "0000".to_string(), "Valid Block".to_string());
        block.mine_block(difficulty).unwrap();

        let result = block.validate("0000", difficulty);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_invalid_hash() {
        let difficulty = 2;
        let mut block = Block::new(1, "0000".to_string(), "Bad Hash".to_string());
        block.mine_block(difficulty).unwrap();
        block.hash = "0001234_fake_hash".to_string();

        let result = block.validate("0000", difficulty);
        match result {
            Err(Error::BlockHasInvalidHash(1, _)) => {}
            v => panic!("Expected error BlockHasInvalidHash, actual {v:?}"),
        }
    }

    #[test]
    fn test_validate_invalid_difficulty() {
        let mut block = Block::new(1, "0000".to_string(), "Low difficulty".to_string());
        block.mine_block(1).unwrap();

        let result = block.validate("0000", 3);
        match result {
            Err(Error::UnsatisfiedHashDifficulty(1, _)) => {}
            v => panic!("Expected error UnsatisfiedHashDifficulty, actual {v:?}"),
        }
    }

    #[test]
    fn test_validate_invalid_previous_hash() {
        let difficulty = 2;
        let mut block = Block::new(1, "abcd".to_string(), "Invalid prev".to_string());
        block.mine_block(difficulty).unwrap();

        let result = block.validate("0000", difficulty);
        match result {
            Err(Error::BlockHasInvalidPreviusBlockHash(1, _, prev)) => {
                assert_eq!(prev, "0000");
            }
            v => panic!("Expected error BlockHasInvalidPreviusBlockHash, actual {v:?}"),
        }
    }
}
