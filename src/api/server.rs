use crate::{
    api::midleware::UuidRequestId,
    block::{Block, Transaction},
    blockchain::Blockchain,
    config::Config,
    errors::Result,
    node::Node,
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
    {
        let str_addr = addr.to_string();
        let peers = {
            let mut node = node.lock().unwrap();
            node.address = str_addr.clone();
            node.peers.iter().cloned().collect::<Vec<_>>()
        };
        notify_peers_about_new_peer(&str_addr, peers).await;
    }
    info!("Starting server at http://{addr}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn notify_peers_about_new_peer(new_peer: &str, peers: Vec<String>) {
    let client = reqwest::Client::new();
    for peer in peers {
        match client
            .post(format!("{peer}/peer"))
            .json(new_peer)
            .send()
            .await
        {
            Ok(resp) => {
                tracing::info!(
                    "New peer {} registration for {}: {}",
                    new_peer,
                    peer,
                    resp.status()
                );
            }
            Err(err) => {
                tracing::warn!(
                    "Failed new peer {} registration for with {}: {}",
                    new_peer,
                    peer,
                    err
                );
            }
        }
    }
}

#[axum::debug_handler]
async fn get_chain(State(node): State<SharedNode>) -> Result<Json<Blockchain>> {
    let chain = node.lock().unwrap().blockchain.clone();
    Ok(Json(chain))
}

#[axum::debug_handler]
async fn register_peer(State(node): State<SharedNode>, Json(data): Json<String>) -> Result<()> {
    let peers: Vec<String>;
    let address: String;
    {
        let mut node = node.lock().unwrap();
        node.register_peer(data.clone());
        peers = node.peers.iter().cloned().collect();
        address = node.address.clone();
    }
    let client = reqwest::Client::new();

    if let Err(err) = client
        .post(format!("{data}/peer"))
        .json(&address)
        .send()
        .await
    {
        tracing::warn!("Failed to sync with {}: {}", &data, err);
    }

    for peer in peers {
        if peer.eq(&data) {
            continue;
        }
        match client.post(format!("{peer}/peer")).json(&data).send().await {
            Ok(resp) => {
                tracing::info!("Synced with peer {}: {}", &peer, resp.status());
            }
            Err(err) => {
                tracing::warn!("Failed to sync with {}: {}", &peer, err);
            }
        }
    }

    Ok(())
}

#[axum::debug_handler]
async fn add_block(
    State(node): State<SharedNode>,
    Json(data): Json<Vec<Transaction>>,
) -> Result<Json<Block>> {
    let peers: Vec<String>;
    let blockchain: Blockchain;
    let block = {
        let mut node = node.lock().unwrap();
        let block = node.add_block(data)?.clone();
        info!("Block mined and added");
        peers = node.peers.iter().cloned().collect();
        blockchain = node.blockchain.clone();
        block
    };

    let client = reqwest::Client::new();
    for peer in peers {
        match client
            .post(format!("{peer}/sync"))
            .json(&blockchain)
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

    Ok(Json(block))
}

#[axum::debug_handler]
async fn sync_chain(
    State(node): State<SharedNode>,
    Json(incoming_chain): Json<Blockchain>,
) -> Result<String> {
    let mut node = node.lock().unwrap();

    tracing::info!(
        "Attempting to sync with incoming chain (length: {})",
        incoming_chain.blocks().len()
    );

    if node.replace_chain(incoming_chain)? {
        tracing::info!("Chain replaced successfully");
        Ok("Chain synced".into())
    } else {
        tracing::warn!("Incoming chain rejected");
        Ok("Incoming chain shorter".into())
    }
}
