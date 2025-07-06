mod api;
mod block;
mod blockchain;
mod errors;
mod node;

use std::sync::{Arc, Mutex};

use errors::Result;
use node::Node;
use tracing_subscriber::EnvFilter;

use crate::api::start_http_server;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        // .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug")),
        )
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE)
        .compact()
        .init();

    tracing::info!("Logger initialized");

    let node = Arc::new(Mutex::new(Node::new("A", 4)?));

    start_http_server(node).await;
    Ok(())
}
