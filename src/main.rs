mod block;

use block::Block;

fn main() {
    let block = Block::new(0, "0".to_string(), "Genesis Block".to_string());

    println!("{:?}", block);
}
