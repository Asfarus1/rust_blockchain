use crate::block::Block;

pub struct Blockchain {
    pub chain: Vec<Block>,
    difficalty: usize,
}

impl Blockchain {
    pub fn new(difficalty: usize) -> Self {
        let mut genesis_block = Block::new(0, "0".to_string(), "Genesis Block".to_string());
        genesis_block.mine_block(difficalty);
        Self {
            chain: vec![genesis_block],
            difficalty,
        }
    }

    pub fn latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn add_block(&mut self, data: String) {
        let latest_block = self.latest_block();
        let mut new_block = Block::new(latest_block.index + 1, latest_block.hash.clone(), data);
        new_block.mine_block(self.difficalty);
        self.chain.push(new_block);
    }
}
