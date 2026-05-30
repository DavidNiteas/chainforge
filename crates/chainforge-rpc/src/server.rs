//! JSON-RPC HTTP 服务器。

use std::sync::Arc;

use axum::extract::State;
use axum::response::Json;
use axum::routing::post;
use axum::Router;
use chainforge_evm::{Address, EvmExecutor, InMemoryEvmState, U256};
use chainforge_storage::{InMemoryStorage, StorageEngine};
use serde_json::json;
use tokio::sync::RwLock;

use crate::types::{RpcRequest, RpcResponse};
use crate::ws::EventBus;

/// RPC 状态。
pub struct RpcState {
    pub chain_id: u64,
    pub mempool: RwLock<chainforge_mempool::Mempool>,
    pub evm_state: RwLock<InMemoryEvmState>,
    pub storage: InMemoryStorage,
    pub event_bus: EventBus,
}

impl RpcState {
    pub fn new(chain_id: u64) -> Self {
        RpcState {
            chain_id,
            mempool: RwLock::new(chainforge_mempool::Mempool::new()),
            evm_state: RwLock::new(InMemoryEvmState::new()),
            storage: InMemoryStorage::new(),
            event_bus: EventBus::default(),
        }
    }
}

/// 创建 RPC 路由。
pub fn routes() -> Router<Arc<RpcState>> {
    Router::new()
        .route("/", post(rpc_handler))
        .route("/ws", axum::routing::get(crate::ws::ws_handler))
}

/// RPC 请求处理器。
async fn rpc_handler(
    State(state): State<Arc<RpcState>>,
    Json(req): Json<RpcRequest>,
) -> Json<RpcResponse> {
    let id = req.id.clone();
    let response = match req.method.as_str() {
        "eth_chainId" => RpcResponse::success(id, json!(format!("0x{:x}", state.chain_id))),
        "net_version" => RpcResponse::success(id, json!(state.chain_id.to_string())),
        "eth_sendRawTransaction" => handle_send_raw_transaction(state, req.params, id).await,
        "eth_getBalance" => handle_get_balance(state, req.params, id).await,
        "eth_getTransactionCount" => handle_get_transaction_count(state, req.params, id).await,
        "eth_getBlockByNumber" => handle_get_block_by_number(state, req.params, id).await,
        "eth_getBlockByHash" => handle_get_block_by_hash(state, req.params, id).await,
        "eth_call" => handle_call(state, req.params, id).await,
        "eth_getCode" => handle_get_code(state, req.params, id).await,
        "eth_blockNumber" => RpcResponse::success(id, json!("0x0")),
        _ => RpcResponse::error(id, -32601, format!("Method not found: {}", req.method)),
    };
    Json(response)
}

fn parse_address(param: &serde_json::Value) -> Option<Address> {
    let s = param.as_str()?;
    let hex = s.trim_start_matches("0x");
    let bytes = hex::decode(hex).ok()?;
    if bytes.len() != 20 {
        return None;
    }
    let mut arr = [0u8; 20];
    arr.copy_from_slice(&bytes);
    Some(Address::new(arr))
}

fn u256_to_hex(value: U256) -> String {
    if value.is_zero() {
        "0x0".to_string()
    } else {
        format!("0x{:x}", value)
    }
}

fn u64_to_hex(value: u64) -> String {
    format!("0x{:x}", value)
}

async fn handle_send_raw_transaction(
    state: Arc<RpcState>,
    params: Vec<serde_json::Value>,
    id: serde_json::Value,
) -> RpcResponse {
    if params.is_empty() {
        return RpcResponse::error(id, -32602, "Missing params".to_string());
    }

    let tx_hex = params[0].as_str().unwrap_or("");
    let tx_bytes = match hex::decode(tx_hex.trim_start_matches("0x")) {
        Ok(b) => b,
        Err(_) => return RpcResponse::error(id, -32602, "Invalid transaction data".to_string()),
    };

    match chainforge_core::tx::Transaction::decode_rlp(&tx_bytes) {
        Ok(tx) => {
            let hash = hex::encode(tx.hash());
            let mut mempool = state.mempool.write().await;
            if mempool.is_nonce_valid(&tx) {
                mempool.insert(tx);
            }
            RpcResponse::success(id, json!(format!("0x{}", hash)))
        }
        Err(_) => RpcResponse::error(id, -32602, "Failed to decode transaction".to_string()),
    }
}

async fn handle_get_balance(
    state: Arc<RpcState>,
    params: Vec<serde_json::Value>,
    id: serde_json::Value,
) -> RpcResponse {
    if params.is_empty() {
        return RpcResponse::error(id, -32602, "Missing params".to_string());
    }
    match parse_address(&params[0]) {
        Some(addr) => {
            let evm = state.evm_state.read().await;
            let balance = evm.balance(addr);
            RpcResponse::success(id, json!(u256_to_hex(balance)))
        }
        None => RpcResponse::error(id, -32602, "Invalid address".to_string()),
    }
}

async fn handle_get_transaction_count(
    state: Arc<RpcState>,
    params: Vec<serde_json::Value>,
    id: serde_json::Value,
) -> RpcResponse {
    if params.is_empty() {
        return RpcResponse::error(id, -32602, "Missing params".to_string());
    }
    match parse_address(&params[0]) {
        Some(addr) => {
            let evm = state.evm_state.read().await;
            let nonce = evm.nonce(addr);
            RpcResponse::success(id, json!(u64_to_hex(nonce)))
        }
        None => RpcResponse::error(id, -32602, "Invalid address".to_string()),
    }
}

async fn handle_get_block_by_number(
    state: Arc<RpcState>,
    params: Vec<serde_json::Value>,
    id: serde_json::Value,
) -> RpcResponse {
    if params.is_empty() {
        return RpcResponse::error(id, -32602, "Missing params".to_string());
    }
    let number_str = params[0].as_str().unwrap_or("latest");
    let key = format!("block:number:{}", number_str);
    match state.storage.get(key.as_bytes()).await {
        Ok(Some(data)) => match chainforge_core::block::Block::decode_rlp(&data) {
            Ok(block) => RpcResponse::success(id, json!({
                "number": u64_to_hex(block.header.number),
                "hash": format!("0x{}", hex::encode(block.header.hash())),
                "parentHash": format!("0x{}", hex::encode(block.header.parent_hash)),
                "timestamp": u64_to_hex(block.header.timestamp),
            })),
            Err(_) => RpcResponse::success(id, serde_json::Value::Null),
        },
        _ => RpcResponse::success(id, serde_json::Value::Null),
    }
}

async fn handle_get_block_by_hash(
    state: Arc<RpcState>,
    params: Vec<serde_json::Value>,
    id: serde_json::Value,
) -> RpcResponse {
    if params.is_empty() {
        return RpcResponse::error(id, -32602, "Missing params".to_string());
    }
    let hash_hex = params[0].as_str().unwrap_or("").trim_start_matches("0x");
    let key = format!("block:hash:{}", hash_hex);
    match state.storage.get(key.as_bytes()).await {
        Ok(Some(data)) => match chainforge_core::block::Block::decode_rlp(&data) {
            Ok(block) => RpcResponse::success(id, json!({
                "number": u64_to_hex(block.header.number),
                "hash": format!("0x{}", hex::encode(block.header.hash())),
                "parentHash": format!("0x{}", hex::encode(block.header.parent_hash)),
                "timestamp": u64_to_hex(block.header.timestamp),
            })),
            Err(_) => RpcResponse::success(id, serde_json::Value::Null),
        },
        _ => RpcResponse::success(id, serde_json::Value::Null),
    }
}

async fn handle_call(
    state: Arc<RpcState>,
    params: Vec<serde_json::Value>,
    id: serde_json::Value,
) -> RpcResponse {
    if params.is_empty() {
        return RpcResponse::error(id, -32602, "Missing params".to_string());
    }

    let call_obj = match params[0].as_object() {
        Some(o) => o,
        None => return RpcResponse::error(id, -32602, "Invalid call object".to_string()),
    };

    let from = call_obj
        .get("from")
        .and_then(parse_address)
        .unwrap_or_else(|| Address::new([0u8; 20]));
    let to = match call_obj.get("to").and_then(parse_address) {
        Some(addr) => addr,
        None => return RpcResponse::error(id, -32602, "Missing 'to' address".to_string()),
    };
    let data = call_obj
        .get("data")
        .and_then(|v| v.as_str())
        .map(|s| hex::decode(s.trim_start_matches("0x")).unwrap_or_default())
        .unwrap_or_default();
    let value = call_obj
        .get("value")
        .and_then(|v| v.as_str())
        .and_then(|s| U256::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or_default();

    let db = {
        let evm = state.evm_state.read().await;
        evm.clone()
    };

    let mut executor = EvmExecutor::new(db);
    match executor.call(from, to, data, value) {
        Ok(result) => {
            let output = result.output().unwrap_or_default();
            RpcResponse::success(id, json!(format!("0x{}", hex::encode(output))))
        }
        Err(e) => RpcResponse::error(id, -32000, format!("EVM call failed: {}", e)),
    }
}

async fn handle_get_code(
    state: Arc<RpcState>,
    params: Vec<serde_json::Value>,
    id: serde_json::Value,
) -> RpcResponse {
    if params.is_empty() {
        return RpcResponse::error(id, -32602, "Missing params".to_string());
    }
    match parse_address(&params[0]) {
        Some(addr) => {
            let evm = state.evm_state.read().await;
            let code = evm.code(addr);
            RpcResponse::success(id, json!(format!("0x{}", hex::encode(code))))
        }
        None => RpcResponse::error(id, -32602, "Invalid address".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    fn app() -> Router {
        let state = Arc::new(RpcState::new(1337));
        routes().with_state(state)
    }

    #[tokio::test]
    async fn test_eth_chain_id() {
        let app = app();
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"jsonrpc":"2.0","method":"eth_chainId","params":[],"id":1}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_eth_send_raw_transaction() {
        let app = app();
        // 构造一笔简单的 RLP 编码交易
        let tx = chainforge_core::tx::Transaction {
            nonce: 0,
            gas_price: 10,
            gas_limit: 21000,
            to: Some([1u8; 20]),
            value: 100,
            data: vec![],
            v: 27,
            r: [0u8; 32],
            s: [0u8; 32],
        };
        let tx_hex = format!("0x{}", hex::encode(tx.encode_rlp()));
        let body = format!(
            "{{\"jsonrpc\":\"2.0\",\"method\":\"eth_sendRawTransaction\",\"params\":[\"{}\"],\"id\":1}}",
            tx_hex
        );

        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_unknown_method() {
        let app = app();
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"jsonrpc":"2.0","method":"eth_unknown","params":[],"id":1}"#))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_eth_get_balance() {
        let state = Arc::new(RpcState::new(1337));
        {
            let mut evm = state.evm_state.write().await;
            evm.set_balance(Address::new([0xAAu8; 20]), U256::from(12345));
        }
        let app = routes().with_state(state);

        let body = r#"{"jsonrpc":"2.0","method":"eth_getBalance","params":["0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","latest"],"id":1}"#;
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["result"].as_str().unwrap(), "0x3039");
    }

    #[tokio::test]
    async fn test_eth_get_transaction_count() {
        let state = Arc::new(RpcState::new(1337));
        {
            let mut evm = state.evm_state.write().await;
            evm.set_nonce(Address::new([0xBBu8; 20]), 42);
        }
        let app = routes().with_state(state);

        let body = r#"{"jsonrpc":"2.0","method":"eth_getTransactionCount","params":["0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","latest"],"id":1}"#;
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json["result"].as_str().unwrap(), "0x2a");
    }

    #[tokio::test]
    async fn test_eth_call() {
        let state = Arc::new(RpcState::new(1337));
        {
            let mut evm = state.evm_state.write().await;
            evm.set_balance(Address::new([0xCCu8; 20]), U256::from(10000));
        }
        let app = routes().with_state(state);

        let body = r#"{"jsonrpc":"2.0","method":"eth_call","params":[{"from":"0xcccccccccccccccccccccccccccccccccccccccc","to":"0xdddddddddddddddddddddddddddddddddddddddd","data":"0x","value":"0x0"},"latest"],"id":1}"#;
        let req = Request::builder()
            .method("POST")
            .uri("/")
            .header("content-type", "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
