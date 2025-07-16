mod common;

use reqwest::Client;
use rust_blockchain::block::Transaction;
use rust_blockchain::blockchain::Blockchain;
use rust_blockchain::config::Config;
use rust_blockchain::{errors::Result, node::Node};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::task;

#[tokio::test]
async fn test_manual_node_sync_between_two_servers() -> Result<()> {
    common::init_tracing();

    let conf_a = Config { port: 3001 };
    let conf_b = Config { port: 3002 };
    let node_a = Arc::new(Mutex::new(Node::new("A", 2)?));
    let node_b = Arc::new(Mutex::new(Node::new("B", 2)?));

    let _ = task::spawn(async move {
        rust_blockchain::api::start_http_server(node_a, conf_a)
            .await
            .unwrap();
    });

    let _ = task::spawn(async move {
        rust_blockchain::api::start_http_server(node_b, conf_b)
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(500)).await;

    let client = Client::new();

    let tx = Transaction {
        from: "A".into(),
        to: "B".into(),
        amount: 1,
    };
    let res = client
        .post("http://localhost:3001/add_block")
        .json(&vec![tx.clone()])
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());

    let chain_a: Blockchain = client
        .get("http://localhost:3001/chain")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let res = client
        .post("http://localhost:3002/sync")
        .json(&chain_a)
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());

    let chain_b: Blockchain = client
        .get("http://localhost:3002/chain")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(chain_a, chain_b);
    assert_eq!(
        chain_b.blocks().last().unwrap().transactions,
        vec![tx.clone()]
    );

    Ok(())
}

#[tokio::test]
async fn test_autosync_between_two_servers() -> Result<()> {
    common::init_tracing();

    let conf_a = Config { port: 3003 };
    let conf_b = Config { port: 3004 };
    let node_a = Arc::new(Mutex::new(Node::new("A", 2)?));
    let node_b = Arc::new(Mutex::new(Node::new("B", 2)?));

    let _ = task::spawn(async move {
        rust_blockchain::api::start_http_server(node_a, conf_a)
            .await
            .unwrap();
    });

    let _ = task::spawn(async move {
        rust_blockchain::api::start_http_server(node_b, conf_b)
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(500)).await;

    let client = Client::new();

    let tx = Transaction {
        from: "A".into(),
        to: "B".into(),
        amount: 1,
    };
    let res = client
        .post("http://localhost:3003/peer")
        .json("http://localhost:3004")
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());

    let res = client
        .post("http://localhost:3003/add_block")
        .json(&vec![tx.clone()])
        .send()
        .await
        .unwrap();
    assert!(res.status().is_success());

    let chain_b: Blockchain = client
        .get("http://localhost:3004/chain")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let last_block_txs = &chain_b.blocks().last().unwrap().transactions;
    assert_eq!(last_block_txs, &vec![tx]);

    Ok(())
}

#[tokio::test]
async fn test_get_balance() {
    common::init_tracing();

    let conf_a = Config { port: 3005 };
    let node_a = Arc::new(Mutex::new(Node::new("A", 2).unwrap()));

    let _ = task::spawn(async move {
        rust_blockchain::api::start_http_server(node_a, conf_a)
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(500)).await;

    let client = Client::new();

    let txs = vec![
        Transaction {
            from: "A".into(),
            to: "B".into(),
            amount: 100,
        },
        Transaction {
            from: "A".into(),
            to: "B".into(),
            amount: 10,
        },
        Transaction {
            from: "B".into(),
            to: "A".into(),
            amount: 50,
        },
    ];
    let _ = client
        .post("http://localhost:3005/add_block")
        .json(&txs)
        .send()
        .await
        .unwrap();

    let balance_b: i64 = client
        .get("http://localhost:3005/balance/B")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(balance_b, 60);

    let balance_c: i64 = client
        .get("http://localhost:3005/balance/C")
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(balance_c, 0);
}
