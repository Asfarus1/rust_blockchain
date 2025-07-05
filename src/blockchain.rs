use crate::block::Block;
use crate::errors::{Error, Result};

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub difficulty: usize,
}

impl Blockchain {
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

    pub fn add_block(&mut self, data: String) -> Result<()> {
        let latest_block = self.latest_block()?;
        let mut new_block = Block::new(latest_block.index + 1, latest_block.hash.clone(), data);
        new_block.mine_block(self.difficulty)?;
        self.chain.push(new_block);
        Ok(())
    }

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
