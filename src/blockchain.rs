use crate::block::Block;

pub struct Blockchain {
    pub chain: Vec<Block>,
    difficulty: usize,
}

impl Blockchain {
    pub fn new(difficulty: usize) -> Self {
        let mut genesis_block = Block::new(0, "0".to_string(), "Genesis Block".to_string());
        genesis_block.mine_block(difficulty);
        Self {
            chain: vec![genesis_block],
            difficulty,
        }
    }

    pub fn latest_block(&self) -> &Block {
        self.chain.last().unwrap()
    }

    pub fn add_block(&mut self, data: String) {
        let latest_block = self.latest_block();
        let mut new_block = Block::new(latest_block.index + 1, latest_block.hash.clone(), data);
        new_block.mine_block(self.difficulty);
        self.chain.push(new_block);
    }

    pub fn validate_chain(&self) -> bool {
        let mut it = self.chain.iter();
        let Some(mut prev_block) = it.next() else {
            println!("Invalid chain: can't be empty");
            return false;
        };
        while let Some(block) = it.next() {
            if block.hash != block.compute_hash() {
                println!(
                    "Invalid chain: block with index {} has invalid hash {}",
                    block.index, block.hash
                );
                return false;
            }
            if block.hash[..self.difficulty] != "0".repeat(self.difficulty) {
                println!(
                    "Invalid chain: block with index {} doesn't satisfy difficulty {}",
                    block.index, block.hash
                );
                return false;
            }
            if block.previous_hash != prev_block.hash {
                println!(
                    "Invalid chain: block with index {} has previous hash {}",
                    block.index, block.previous_hash
                );
                return false;
            }
            prev_block = block;
        }
        println!("Chain is valid!");
        true
    }
}
