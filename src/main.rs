mod block;
mod blockchain;
mod errors;
mod node;

use errors::Result;
use node::Node;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .init();

    tracing::info!("Logger initialized");
    tracing::debug!("Logger initialized");

    let mut node_a = Node::new("A", 4)?;
    let mut node_b = Node::new("B", 4)?;

    node_a.add_block("Tx1 from A")?;
    node_a.add_block("Tx2 from A")?;

    println!("\nNode B syncs from A");
    node_b.replace_chain(node_a.blockchain.chain.clone())?;

    node_a.add_block("Tx3 from A")?;

    println!("\nNode B tries to sync outdated chain (should be rejected)");
    node_b.replace_chain(node_b.blockchain.chain.clone())?;

    println!("\nFinal chains:");
    node_a.print_chain();
    node_b.print_chain();
    Ok(())
}
