use crate::block::{Block, Transaction};
use crate::errors::{Error, Result};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Blockchain {
    chain: Vec<Block>,
    difficulty: usize,
}

impl Blockchain {
    #[instrument(name = "create_new_blockchain", level = "debug")]
    pub fn new(difficulty: usize) -> Result<Self> {
        let mut genesis_block = Block::new(0, "0".to_string(), vec![]);
        genesis_block.mine_block(difficulty)?;
        Ok(Self {
            chain: vec![genesis_block],
            difficulty,
        })
    }

    pub fn blocks(&self) -> &[Block] {
        &self.chain[..]
    }

    pub fn get_balance(&self, address: &str) -> i64 {
        self.chain
            .iter()
            .flat_map(|b| b.transactions.iter())
            .filter(|t| t.from == address || t.to == address)
            .fold(0i64, |acc, t| {
                if t.from == address {
                    acc - t.amount
                } else {
                    acc + t.amount
                }
            })
    }

    #[instrument(skip(self), level = "debug", name = "add_block_to_blockchain")]
    pub fn add_block(&mut self, transactions: Vec<Transaction>) -> Result<&Block> {
        let latest_block = self.chain.last().ok_or(Error::ChainIsEmpty)?;
        let mut new_block = Block::new(
            latest_block.index + 1,
            latest_block.hash.clone(),
            transactions,
        );
        new_block.mine_block(self.difficulty)?;
        self.chain.push(new_block);
        Ok(self.blocks().last().unwrap())
    }

    #[allow(unused)]
    #[instrument(level = "debug")]
    pub fn validate(&self) -> Result<()> {
        let mut it = self.chain.iter();
        let mut prev_block = it.next().ok_or(Error::ChainIsEmpty)?;
        for block in it {
            block.validate(&prev_block.hash, self.difficulty)?;
            prev_block = block;
        }
        Ok(())
    }

    #[allow(unused)]
    #[instrument(skip_all, level = "info")]
    pub fn replace_chain(&mut self, other: Blockchain) -> Result<bool> {
        if other.chain.len() <= self.chain.len() {
            return Ok(false);
        }
        if let Err(e) = other.validate() {
            error!("Failed to replace chain {:?}", e);
            Err(e)?;
        }
        *self = other;

        Ok(true)
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
        assert_eq!(genesis.transactions, vec![]);
        assert!(genesis.hash.starts_with(&"0".repeat(difficulty)));
    }

    #[test]
    fn test_add_block_appends_new_block() {
        let difficulty = 2;
        let mut blockchain = Blockchain::new(difficulty).unwrap();

        blockchain
            .add_block(vec![Transaction {
                from: "A".to_string(),
                to: "B".to_string(),
                amount: 100,
            }])
            .unwrap();
        blockchain
            .add_block(vec![Transaction {
                from: "B".to_string(),
                to: "C".to_string(),
                amount: 100,
            }])
            .unwrap();

        assert_eq!(blockchain.chain.len(), 3);
        assert_eq!(
            blockchain.chain[2].transactions,
            vec![Transaction {
                from: "B".to_string(),
                to: "C".to_string(),
                amount: 100,
            }]
        );
        assert_eq!(blockchain.chain[2].index, 2);
        assert_eq!(blockchain.chain[2].previous_hash, blockchain.chain[1].hash);
    }

    #[test]
    fn test_latest_block_returns_last_block() {
        let difficulty = 2;
        let mut blockchain = Blockchain::new(difficulty).unwrap();
        blockchain
            .add_block(vec![Transaction {
                from: "A".to_string(),
                to: "B".to_string(),
                amount: 100,
            }])
            .unwrap();

        let latest = blockchain.blocks().last().unwrap();
        assert_eq!(
            latest.transactions,
            vec![Transaction {
                from: "A".to_string(),
                to: "B".to_string(),
                amount: 100,
            }]
        );
    }

    #[test]
    fn test_validate_chain_success() {
        let difficulty = 2;
        let mut blockchain = Blockchain::new(difficulty).unwrap();
        blockchain
            .add_block(vec![Transaction {
                from: "A".to_string(),
                to: "B".to_string(),
                amount: 100,
            }])
            .unwrap();
        blockchain
            .add_block(vec![Transaction {
                from: "B".to_string(),
                to: "C".to_string(),
                amount: 100,
            }])
            .unwrap();

        let result = blockchain.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_chain_with_tampered_block_hash() {
        let difficulty = 2;
        let mut blockchain = Blockchain::new(difficulty).unwrap();
        blockchain
            .add_block(vec![Transaction {
                from: "A".to_string(),
                to: "B".to_string(),
                amount: 100,
            }])
            .unwrap();
        blockchain
            .add_block(vec![Transaction {
                from: "B".to_string(),
                to: "C".to_string(),
                amount: 100,
            }])
            .unwrap();

        blockchain.chain[2].hash = "00_bad_hash".to_string();

        let result = blockchain.validate();
        match result {
            Err(Error::BlockHasInvalidHash(index, _)) => assert_eq!(index, 2),
            v => panic!("Expected error BlockHasInvalidHash, actual {v:?}"),
        }
    }

    #[test]
    fn test_validate_chain_with_wrong_difficulty() {
        let difficulty = 2;
        let mut blockchain = Blockchain::new(difficulty).unwrap();
        blockchain
            .add_block(vec![Transaction {
                from: "A".to_string(),
                to: "B".to_string(),
                amount: 100,
            }])
            .unwrap();
        blockchain.difficulty = 4;

        let result = blockchain.validate();
        match result {
            Err(Error::UnsatisfiedHashDifficulty(index, _)) => assert_eq!(index, 1),
            v => panic!("Expected error UnsatisfiedHashDifficulty, actual {v:?}"),
        }
    }

    #[test]
    fn test_validate_chain_with_wrong_previous_hash() {
        let difficulty = 2;
        let mut blockchain = Blockchain::new(difficulty).unwrap();
        blockchain
            .add_block(vec![Transaction {
                from: "A".to_string(),
                to: "B".to_string(),
                amount: 100,
            }])
            .unwrap();

        blockchain.chain[1].previous_hash = "WRONG".to_string();

        let result = blockchain.validate();
        match result {
            Err(Error::BlockHasInvalidPreviusBlockHash(index, _, _)) => assert_eq!(index, 1),
            v => panic!("Expected error BlockHasInvalidPreviusBlockHash, actual {v:?}"),
        }
    }

    #[test]
    fn test_validate_empty_chain_fails() {
        let mut blockchain = Blockchain::new(2).unwrap();
        blockchain.chain = vec![];
        let result = blockchain.validate();
        assert!(matches!(result, Err(Error::ChainIsEmpty)));
    }
}
