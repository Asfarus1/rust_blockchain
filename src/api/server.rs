use crate::{
    api::midleware::UuidRequestId, block::Block, blockchain::Blockchain, config::Config,
    errors::Result, node::Node,
};
use axum::{
    Router,
    extract::{Json, Request, State},
    http::HeaderName,
    routing::{get, post},
};
use std::sync::{Arc, Mutex};
use tower_http::{
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::info;

pub type SharedNode = Arc<Mutex<Node>>;

pub async fn start_http_server(node: SharedNode, conf: Config) -> Result<()> {
    let app = Router::new()
        .route("/chain", get(get_chain))
        .route("/add_block", post(add_block))
        .route("/sync", post(sync_chain))
        .route("/peer", post(register_peer))
        .with_state(node.clone())
        .layer(
            TraceLayer::new_for_http().make_span_with(|req: &Request<_>| {
                let request_id = req
                    .headers()
                    .get("x-request-id")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("UNKNOWN");

                tracing::debug_span!(
                    "http_request",
                    method = ?req.method(),
                    uri = %req.uri(),
                    request_id = %request_id,
                )
            }),
        )
        .layer(PropagateRequestIdLayer::new(HeaderName::from_static(
            "x-request-id",
        )))
        .layer(SetRequestIdLayer::x_request_id(UuidRequestId));

    let addr = format!("127.0.0.1:{}", conf.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let addr = listener.local_addr()?;
    info!("Starting server at http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

#[axum::debug_handler]
async fn get_chain(State(node): State<SharedNode>) -> Result<Json<Blockchain>> {
    let chain = node.lock().unwrap().blockchain.clone();
    Ok(Json(chain))
}

#[axum::debug_handler]
async fn register_peer(State(node): State<SharedNode>, Json(data): Json<String>) -> Result<()> {
    node.lock().unwrap().register_peer(data);
    Ok(())
}

#[axum::debug_handler]
async fn add_block(
    State(node): State<SharedNode>,
    Json(data): Json<String>,
) -> Result<Json<Block>> {
    let peers: Vec<String>;
    let chain: Vec<Block>;
    {
        let mut node = node.lock().unwrap();
        node.add_block(&data)?;
        info!("Block mined and added");
        peers = node.peers.iter().cloned().collect();
        chain = node.blockchain.chain.clone();
    }

    let client = reqwest::Client::new();
    for peer in peers {
        match client
            .post(format!("{peer}/sync"))
            .json(&chain)
            .send()
            .await
        {
            Ok(resp) => {
                tracing::info!("Synced with peer {}: {}", peer, resp.status());
            }
            Err(err) => {
                tracing::warn!("Failed to sync with {}: {}", peer, err);
            }
        }
    }
    let block = chain.last().unwrap();
    Ok(Json(block.to_owned()))
}

#[axum::debug_handler]
async fn sync_chain(
    State(node): State<SharedNode>,
    Json(incoming_chain): Json<Vec<Block>>,
) -> Result<String> {
    let mut node = node.lock().unwrap();

    tracing::info!(
        "Attempting to sync with incoming chain (length: {})",
        incoming_chain.len()
    );

    if node.replace_chain(incoming_chain)? {
        tracing::info!("Chain replaced successfully");
        Ok("Chain synced".into())
    } else {
        tracing::warn!("Incoming chain rejected");
        Ok("Incoming chain shorter".into())
    }
}
