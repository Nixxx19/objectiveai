//! JSON-RPC 2.0 envelope types used by the MCP transport.

/// JSON-RPC 2.0 response envelope.
#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum JsonRpcResponse<T> {
    Success {
        #[allow(dead_code)]
        jsonrpc: String,
        #[allow(dead_code)]
        id: serde_json::Value,
        result: T,
    },
    Error {
        #[allow(dead_code)]
        jsonrpc: String,
        #[allow(dead_code)]
        id: serde_json::Value,
        error: JsonRpcError,
    },
}

/// JSON-RPC 2.0 error object.
#[derive(serde::Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 notification (no `id` field).
#[derive(serde::Deserialize)]
pub struct JsonRpcNotification {
    #[allow(dead_code)]
    pub jsonrpc: String,
    pub method: String,
}
