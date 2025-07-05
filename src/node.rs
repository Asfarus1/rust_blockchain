use crate::blockchain::Blockchain;
use crate::errors::Result;

pub struct Node {
    pub name: String,
    pub blockchain: Blockchain,
}

impl Node {
    pub fn new(name: &str, difficulty: usize) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            blockchain: Blockchain::new(difficulty)?,
        })
    }

    pub fn add_block(&mut self, data: &str) -> Result<()> {
        println!("Node {} mining block with data: {}", self.name, data);
        self.blockchain.add_block(data.to_string())
    }

    pub fn replace_chain(&mut self, new_chain: Vec<crate::block::Block>) {
        if new_chain.len() > self.blockchain.chain.len()
            && Blockchain::validate_chain(&new_chain, self.blockchain.difficulty).is_ok()
        {
            println!("Node {} replacing chain with longer valid chain", self.name);
            self.blockchain.chain = new_chain;
        } else {
            println!("Node {} rejected new chain", self.name);
        }
    }

    pub fn print_chain(&self) {
        println!("Chain of node {}:", self.name);
        for block in &self.blockchain.chain {
            println!("{block}");
        }
        println!("-----------------------------");
    }
}
