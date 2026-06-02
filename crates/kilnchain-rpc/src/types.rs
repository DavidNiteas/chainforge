//! JSON-RPC 类型定义。

use serde::{Deserialize, Serialize};

/// JSON-RPC 请求。
#[derive(Clone, Debug, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Vec<serde_json::Value>,
    pub id: serde_json::Value,
}

/// JSON-RPC 响应。
#[derive(Clone, Debug, Serialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: serde_json::Value,
}

/// JSON-RPC 错误。
#[derive(Clone, Debug, Serialize)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl RpcResponse {
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn error(id: serde_json::Value, code: i64, message: String) -> Self {
        RpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}
