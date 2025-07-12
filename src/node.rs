use std::collections::HashSet;

use crate::errors::Result;
use crate::{block::Block, blockchain::Blockchain};
use tracing::{error, info, instrument};

pub struct Node {
    pub address: String,
    pub name: String,
    pub blockchain: Blockchain,
    pub peers: HashSet<String>,
}

impl Node {
    #[instrument(name = "create_new_node", level = "info")]
    pub fn new(name: &str, difficulty: usize) -> Result<Self> {
        Ok(Self {
            address: String::new(),
            name: name.to_string(),
            blockchain: Blockchain::new(difficulty)?,
            peers: HashSet::new(),
        })
    }

    #[instrument(skip(self), fields(node_name = self.name), name = "add_block_to_node", level = "info")]
    pub fn add_block(&mut self, data: &str) -> Result<&Block> {
        self.blockchain.add_block(data.to_string())
    }

    #[allow(unused)]
    #[instrument(skip_all, fields(node_name = self.name), level = "info")]
    pub fn replace_chain(&mut self, other: Blockchain) -> Result<bool> {
        let replaced = self.blockchain.replace_chain(other);
        match replaced {
            Err(ref e) => error!("Failed to replace chain {:?}", e),
            Ok(false) => info!("Node {} new chain is not longer", self.name),
            _ => {}
        }
        replaced
    }

    #[allow(unused)]
    #[instrument(skip(self), fields(node_name = self.name), level = "info")]
    pub fn register_peer(&mut self, peer: String) -> bool {
        self.peers.insert(peer)
    }

    #[allow(unused)]
    pub fn print_chain(&self) {
        println!("Chain of node {}:", self.name);
        for block in self.blockchain.blocks() {
            println!("{block}");
        }
        println!("-----------------------------");
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::blockchain;
    use crate::errors::Error;

    #[test]
    fn test_create_new_node() {
        // Create a new node and ensure it contains the genesis block
        let node = Node::new("Node1", 2).unwrap();
        assert_eq!(node.name, "Node1");
        assert_eq!(node.blockchain.blocks().len(), 1); // only genesis block
    }

    #[test]
    fn test_add_block_to_node() {
        // Add two blocks to the node and verify their data and indexes
        let mut node = Node::new("NodeA", 2).unwrap();
        node.add_block("Test 1").unwrap();
        node.add_block("Test 2").unwrap();

        assert_eq!(node.blockchain.blocks().len(), 3);
        assert_eq!(node.blockchain.blocks()[2].data, "Test 2");
    }

    #[test]
    fn test_replace_chain_success() {
        // Replace current chain with a longer and valid one
        let mut node1 = Node::new("Original", 2).unwrap();
        node1.add_block("Block A").unwrap();

        let mut longer_chain = Blockchain::new(2).unwrap();
        longer_chain.add_block("Block A".to_owned()).unwrap();
        longer_chain.add_block("Block B".to_owned()).unwrap();

        let replaced = node1.replace_chain(longer_chain).unwrap();
        assert!(replaced);
        assert_eq!(node1.blockchain.blocks().len(), 3);
    }

    #[test]
    fn test_replace_chain_fails_if_not_longer() {
        // Attempt to replace the chain with a shorter one should fail
        let mut node1 = Node::new("A", 2).unwrap();
        node1.add_block("Only block").unwrap();

        let shorter_chain = Blockchain::new(2).unwrap();

        let replaced = node1.replace_chain(shorter_chain).unwrap();
        assert!(!replaced); // replacement should be rejected
        assert_eq!(node1.blockchain.blocks().len(), 2); // original chain remains
    }

    #[test]
    fn test_replace_chain_fails_if_invalid() {
        let mut node = Node::new("Node", 2).unwrap();
        node.add_block("Valid block").unwrap();

        let fake_chain: Blockchain = serde_json::from_value(json!({
            "chain":[
                {
                    "index" : 0,
                    "timestamp": 0,
                    "previous_hash": "000",
                    "hash": "000",
                    "data" :"data",
                    "nonce" :0
                },
                {
                    "index" : 1,
                    "timestamp": 0,
                    "previous_hash": "000",
                    "hash": "00056712531",
                    "data" :"fake",
                    "nonce" :0
                },
                {
                    "index" : 1,
                    "timestamp": 0,
                    "previous_hash": "00056712531",
                    "hash": "000",
                    "data" :"fake",
                    "nonce" :0
                },
            ],
            "difficulty": 2
        }))
        .unwrap();

        let result = node.replace_chain(fake_chain);
        match result {
            Err(Error::BlockHasInvalidHash(1, _)) => {}
            v => panic!("Expected error BlockHasInvalidHash, actual {v:?}"),
        }

        assert_eq!(node.blockchain.blocks().len(), 2); // chain remains unchanged
    }

    #[test]
    fn test_print_chain_runs_without_panic() {
        let mut node = Node::new("Printable", 2).unwrap();
        node.add_block("Print me").unwrap();
        node.print_chain(); // smoke test
    }
}
