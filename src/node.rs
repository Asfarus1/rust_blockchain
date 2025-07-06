use crate::blockchain::Blockchain;
use crate::errors::Result;
use tracing::{error, info, instrument};

pub struct Node {
    pub name: String,
    pub blockchain: Blockchain,
}

impl Node {
    #[instrument(name = "create_new_node", level = "info")]
    pub fn new(name: &str, difficulty: usize) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            blockchain: Blockchain::new(difficulty)?,
        })
    }

    #[instrument(skip(self), fields(node_name = self.name), name = "add_block_to_node", level = "info")]
    pub fn add_block(&mut self, data: &str) -> Result<()> {
        self.blockchain.add_block(data.to_string())
    }

    #[allow(unused)]
    #[instrument(skip_all, fields(node_name = self.name, result), level = "info")]
    pub fn replace_chain(&mut self, new_chain: Vec<crate::block::Block>) -> Result<bool> {
        if new_chain.len() <= self.blockchain.chain.len() {
            info!("Node {} new chain is not longer", self.name);
            return Ok(false);
        }
        if let Err(e) = Blockchain::validate_chain(&new_chain, self.blockchain.difficulty) {
            error!("Failed to replace chain {:?}", e);
            Err(e)?;
        }
        self.blockchain.chain = new_chain;

        Ok(true)
    }

    #[allow(unused)]
    pub fn print_chain(&self) {
        println!("Chain of node {}:", self.name);
        for block in &self.blockchain.chain {
            println!("{block}");
        }
        println!("-----------------------------");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain;
    use crate::errors::Error;

    #[test]
    fn test_create_new_node() {
        // Create a new node and ensure it contains the genesis block
        let node = Node::new("Node1", 2).unwrap();
        assert_eq!(node.name, "Node1");
        assert_eq!(node.blockchain.chain.len(), 1); // only genesis block
    }

    #[test]
    fn test_add_block_to_node() {
        // Add two blocks to the node and verify their data and indexes
        let mut node = Node::new("NodeA", 2).unwrap();
        node.add_block("Test 1").unwrap();
        node.add_block("Test 2").unwrap();

        assert_eq!(node.blockchain.chain.len(), 3);
        assert_eq!(node.blockchain.chain[2].data, "Test 2");
    }

    #[test]
    fn test_replace_chain_success() {
        // Replace current chain with a longer and valid one
        let mut node1 = Node::new("Original", 2).unwrap();
        node1.add_block("Block A").unwrap();

        let mut node2 = Node::new("Donor", 2).unwrap();
        node2.add_block("Block A").unwrap();
        node2.add_block("Block B").unwrap();

        let longer_chain = node2.blockchain.chain.clone();

        let replaced = node1.replace_chain(longer_chain).unwrap();
        assert!(replaced);
        assert_eq!(node1.blockchain.chain.len(), 3);
    }

    #[test]
    fn test_replace_chain_fails_if_not_longer() {
        // Attempt to replace the chain with a shorter one should fail
        let mut node1 = Node::new("A", 2).unwrap();
        node1.add_block("Only block").unwrap();

        let node2 = Node::new("B", 2).unwrap(); // only genesis block

        let shorter_chain = node2.blockchain.chain.clone();

        let replaced = node1.replace_chain(shorter_chain).unwrap();
        assert!(!replaced); // replacement should be rejected
        assert_eq!(node1.blockchain.chain.len(), 2); // original chain remains
    }

    #[test]
    fn test_replace_chain_fails_if_invalid() {
        let mut node = Node::new("Node", 2).unwrap();
        node.add_block("Valid block").unwrap();

        let mut fake_chain = blockchain::Blockchain::new(2).unwrap();
        fake_chain.add_block("Valid block".to_string()).unwrap();
        fake_chain.add_block("Tanpered block".to_string()).unwrap();
        fake_chain.chain[2].hash = "00_tampered_hash".to_string();

        let result = node.replace_chain(fake_chain.chain);
        match result {
            Err(Error::BlockHasInvalidHash(2, _)) => {}
            v => panic!("Expected error BlockHasInvalidHash, actual {v:?}"),
        }

        assert_eq!(node.blockchain.chain.len(), 2); // chain remains unchanged
    }

    #[test]
    fn test_print_chain_runs_without_panic() {
        let mut node = Node::new("Printable", 2).unwrap();
        node.add_block("Print me").unwrap();
        node.print_chain(); // smoke test
    }
}
