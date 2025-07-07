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

pub async fn start_http_server(node: SharedNode, conf: Config) {
    let app = Router::new()
        .route("/chain", get(get_chain))
        .route("/add_block", post(add_block))
        .route("/sync", post(sync_chain))
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

    dbg!(&conf);
    let addr = format!("127.0.0.1:{}", conf.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("Starting server at http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn get_chain(State(node): State<SharedNode>) -> Result<Json<Blockchain>> {
    let chain = node.lock().unwrap().blockchain.clone();
    Ok(Json(chain))
}

#[axum::debug_handler]
async fn add_block(State(node): State<SharedNode>, Json(data): Json<String>) -> Result<String> {
    info!("Incoming block data: {}", data);
    let mut node = node.lock().unwrap();
    node.add_block(&data)?;
    Ok("Block mined and added".to_string())
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
