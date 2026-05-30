//! WebSocket JSON-RPC 订阅。

use std::collections::HashMap;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::response::Response;
use futures_util::{stream::StreamExt, SinkExt};
use serde_json::json;
use tokio::sync::broadcast;

use crate::types::{RpcRequest, RpcResponse};

/// 事件广播器。
#[derive(Clone)]
pub struct EventBus {
    pub new_heads: broadcast::Sender<serde_json::Value>,
    pub pending_txs: broadcast::Sender<serde_json::Value>,
}

impl Default for EventBus {
    fn default() -> Self {
        EventBus {
            new_heads: broadcast::channel(128).0,
            pending_txs: broadcast::channel(128).0,
        }
    }
}

/// WebSocket upgrade handler。
pub async fn ws_handler(
    State(state): State<std::sync::Arc<crate::server::RpcState>>,
    ws: WebSocketUpgrade,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: std::sync::Arc<crate::server::RpcState>) {
    let (ws_tx, mut ws_rx) = socket.split();
    let ws_tx = std::sync::Arc::new(tokio::sync::Mutex::new(ws_tx));
    let mut subs: HashMap<String, tokio::task::JoinHandle<()>> = HashMap::new();
    let mut next_id = 1u64;

    while let Some(Ok(msg)) = ws_rx.next().await {
        if let Message::Text(text) = msg {
            let req: RpcRequest = match serde_json::from_str(&text) {
                Ok(r) => r,
                Err(_) => continue,
            };

            match req.method.as_str() {
                "eth_subscribe" => {
                    let sub_type = req.params.first().and_then(|v| v.as_str()).unwrap_or("");
                    let sub_id = format!("0x{:x}", next_id);
                    next_id += 1;

                    let resp = match sub_type {
                        "newHeads" => {
                            let mut rx = state.event_bus.new_heads.subscribe();
                            let tx = ws_tx.clone();
                            let sub_id_clone = sub_id.clone();
                            let handle = tokio::spawn(async move {
                                while let Ok(val) = rx.recv().await {
                                    let msg = serde_json::to_string(&json!({
                                        "jsonrpc": "2.0",
                                        "method": "eth_subscription",
                                        "params": {
                                            "subscription": &sub_id_clone,
                                            "result": val,
                                        }
                                    }))
                                    .unwrap_or_default();
                                    let _ = tx.lock().await.send(Message::Text(msg)).await;
                                }
                            });
                            subs.insert(sub_id.clone(), handle);
                            RpcResponse::success(req.id, json!(sub_id))
                        }
                        "newPendingTransactions" => {
                            let mut rx = state.event_bus.pending_txs.subscribe();
                            let tx = ws_tx.clone();
                            let sub_id_clone = sub_id.clone();
                            let handle = tokio::spawn(async move {
                                while let Ok(val) = rx.recv().await {
                                    let msg = serde_json::to_string(&json!({
                                        "jsonrpc": "2.0",
                                        "method": "eth_subscription",
                                        "params": {
                                            "subscription": &sub_id_clone,
                                            "result": val,
                                        }
                                    }))
                                    .unwrap_or_default();
                                    let _ = tx.lock().await.send(Message::Text(msg)).await;
                                }
                            });
                            subs.insert(sub_id.clone(), handle);
                            RpcResponse::success(req.id, json!(sub_id))
                        }
                        _ => RpcResponse::error(
                            req.id,
                            -32602,
                            format!("Unsupported subscription type: {}", sub_type),
                        ),
                    };

                    let text = serde_json::to_string(&resp).unwrap_or_default();
                    let _ = ws_tx.lock().await.send(Message::Text(text)).await;
                }
                "eth_unsubscribe" => {
                    let sub_id = req.params.first().and_then(|v| v.as_str()).unwrap_or("");
                    if let Some(handle) = subs.remove(sub_id) {
                        handle.abort();
                    }
                    let resp = RpcResponse::success(req.id, json!(true));
                    let text = serde_json::to_string(&resp).unwrap_or_default();
                    let _ = ws_tx.lock().await.send(Message::Text(text)).await;
                }
                _ => {
                    let resp = RpcResponse::error(req.id, -32601, "Method not found".to_string());
                    let text = serde_json::to_string(&resp).unwrap_or_default();
                    let _ = ws_tx.lock().await.send(Message::Text(text)).await;
                }
            }
        }
    }

    // 连接断开时取消所有订阅
    for (_, handle) in subs {
        handle.abort();
    }
}
