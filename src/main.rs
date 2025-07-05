mod block;
mod blockchain;

use blockchain::Blockchain;

fn main() {
    let mut blockchain = Blockchain::new(3);
    blockchain.add_block("First block".to_string());
    blockchain.add_block("Second block".to_string());

    blockchain.validate_chain();
}
