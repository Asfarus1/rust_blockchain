use crate::block::Block;

pub struct Blockchain {
    pub chain: Vec<Block>,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis_block = Block::new(0, "0".to_string(), "Genesis Block".to_string());
        Self {
            chain: vec![genesis_block],
        }
    }

    pub fn latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn add_block(&mut self, data: String) {
        let latest_block = self.latest_block();
        let new_block = Block::new(latest_block.index + 1, latest_block.hash.clone(), data);
        self.chain.push(new_block);
    }
}
