use crate::{api::midleware::UuidRequestId, errors::Error, node::Node};
use axum::{
    Router,
    extract::{Json, Request, State},
    http::HeaderName,
    response::IntoResponse,
    routing::{get, post},
};
use std::sync::{Arc, Mutex};
use tower_http::{
    request_id::{PropagateRequestIdLayer, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing::info;

pub type SharedNode = Arc<Mutex<Node>>;

pub async fn start_http_server(node: SharedNode) {
    let app = Router::new()
        .route("/chain", get(get_chain))
        .route("/add_block", post(add_block))
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

    let addr = "127.0.0.1:3030";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Starting server at http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn get_chain(State(node): State<SharedNode>) -> impl IntoResponse {
    let chain = node.lock().unwrap().blockchain.chain.clone();
    Json(chain)
}

#[axum::debug_handler]
async fn add_block(
    State(node): State<SharedNode>,
    Json(data): Json<String>,
) -> Result<String, Error> {
    info!("Incoming block data: {}", data);
    let mut node = node.lock().unwrap();
    node.add_block(&data)?;
    Ok("Block mined and added".to_string())
}
