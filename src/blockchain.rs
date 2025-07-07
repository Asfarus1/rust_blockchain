use crate::block::Block;
use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
}

impl Blockchain {
    #[instrument(name = "create_new_blockchain", level = "debug")]
    pub fn new(difficulty: usize) -> Result<Self> {
        let mut genesis_block = Block::new(0, "0".to_string(), "Genesis Block".to_string());
        genesis_block.mine_block(difficulty)?;
        Ok(Self {
            chain: vec![genesis_block],
            difficulty,
        })
    }

    pub fn latest_block(&self) -> Result<&Block> {
        self.chain.last().ok_or(Error::ChainIsEmpty)
    }

    #[instrument(skip(self), level = "debug", name = "add_block_to_blockchain")]
    pub fn add_block(&mut self, data: String) -> Result<()> {
        let latest_block = self.latest_block()?;
        let mut new_block = Block::new(latest_block.index + 1, latest_block.hash.clone(), data);
        new_block.mine_block(self.difficulty)?;
        self.chain.push(new_block);
        Ok(())
    }

    #[allow(unused)]
    #[instrument(skip(chain), level = "debug")]
    pub fn validate_chain(chain: &[Block], difficulty: usize) -> Result<()> {
        let mut it = chain.iter();
        let mut prev_block = it.next().ok_or(Error::ChainIsEmpty)?;
        for block in it {
            block.validate(&prev_block.hash, difficulty)?;
            prev_block = block;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::Error;

    #[test]
    fn test_new_blockchain_creates_genesis_block() {
        let difficulty = 2;
        let blockchain = Blockchain::new(difficulty).unwrap();

        assert_eq!(blockchain.chain.len(), 1);
        let genesis = &blockchain.chain[0];
        assert_eq!(genesis.index, 0);
        assert_eq!(genesis.previous_hash, "0");
        assert_eq!(genesis.data, "Genesis Block");
        assert!(genesis.hash.starts_with(&"0".repeat(difficulty)));
    }

    #[test]
    fn test_add_block_appends_new_block() {
        let difficulty = 2;
        let mut blockchain = Blockchain::new(difficulty).unwrap();

        blockchain.add_block("Block 1".to_string()).unwrap();
        blockchain.add_block("Block 2".to_string()).unwrap();

        assert_eq!(blockchain.chain.len(), 3);
        assert_eq!(blockchain.chain[2].data, "Block 2");
        assert_eq!(blockchain.chain[2].index, 2);
        assert_eq!(blockchain.chain[2].previous_hash, blockchain.chain[1].hash);
    }

    #[test]
    fn test_latest_block_returns_last_block() {
        let difficulty = 2;
        let mut blockchain = Blockchain::new(difficulty).unwrap();
        blockchain.add_block("Last Block".to_string()).unwrap();

        let latest = blockchain.latest_block().unwrap();
        assert_eq!(latest.data, "Last Block");
    }

    #[test]
    fn test_validate_chain_success() {
        let difficulty = 2;
        let mut blockchain = Blockchain::new(difficulty).unwrap();
        blockchain.add_block("A".to_string()).unwrap();
        blockchain.add_block("B".to_string()).unwrap();

        let result = Blockchain::validate_chain(&blockchain.chain, difficulty);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_chain_with_tampered_block_hash() {
        let difficulty = 2;
        let mut blockchain = Blockchain::new(difficulty).unwrap();
        blockchain.add_block("X".to_string()).unwrap();
        blockchain.add_block("Y".to_string()).unwrap();

        blockchain.chain[2].hash = "00_bad_hash".to_string();

        let result = Blockchain::validate_chain(&blockchain.chain, difficulty);
        match result {
            Err(Error::BlockHasInvalidHash(index, _)) => assert_eq!(index, 2),
            v => panic!("Expected error BlockHasInvalidHash, actual {v:?}"),
        }
    }

    #[test]
    fn test_validate_chain_with_wrong_difficulty() {
        let difficulty = 2;
        let wrong_difficulty = 4;
        let mut blockchain = Blockchain::new(difficulty).unwrap();
        blockchain.add_block("test".to_string()).unwrap();

        let result = Blockchain::validate_chain(&blockchain.chain, wrong_difficulty);
        match result {
            Err(Error::UnsatisfiedHashDifficulty(index, _)) => assert_eq!(index, 1),
            v => panic!("Expected error UnsatisfiedHashDifficulty, actual {v:?}"),
        }
    }

    #[test]
    fn test_validate_chain_with_wrong_previous_hash() {
        let difficulty = 2;
        let mut blockchain = Blockchain::new(difficulty).unwrap();
        blockchain.add_block("test".to_string()).unwrap();

        blockchain.chain[1].previous_hash = "WRONG".to_string();

        let result = Blockchain::validate_chain(&blockchain.chain, difficulty);
        match result {
            Err(Error::BlockHasInvalidPreviusBlockHash(index, _, _)) => assert_eq!(index, 1),
            v => panic!("Expected error BlockHasInvalidPreviusBlockHash, actual {v:?}"),
        }
    }

    #[test]
    fn test_latest_block_on_empty_chain_fails() {
        let blockchain = Blockchain {
            chain: vec![],
            difficulty: 2,
        };
        let result = blockchain.latest_block();
        assert!(matches!(result, Err(Error::ChainIsEmpty)));
    }

    #[test]
    fn test_validate_empty_chain_fails() {
        let result = Blockchain::validate_chain(&[], 2);
        assert!(matches!(result, Err(Error::ChainIsEmpty)));
    }
}
