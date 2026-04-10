use crate::{Aegis, Policy};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, serde::Deserialize)]
pub struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<serde_json::Value>,
    id: Option<serde_json::Value>,
}

#[derive(Debug, serde::Serialize)]
pub struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: Option<serde_json::Value>,
}

#[derive(Debug, serde::Serialize)]
pub struct JsonRpcError {
    code: i32,
    message: String,
}

pub struct McpServer {
    aegis: Arc<Mutex<Aegis>>,
    policy: Arc<Policy>,
}

impl McpServer {
    pub fn new(aegis: Aegis, policy: Policy) -> Self {
        Self {
            aegis: Arc::new(Mutex::new(aegis)),
            policy: Arc::new(policy),
        }
    }

    pub fn handle_request(&self, body: &str) -> JsonRpcResponse {
        let request: JsonRpcRequest = match serde_json::from_str(body) {
            Ok(r) => r,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                    }),
                    id: None,
                };
            }
        };

        let response = match request.method.as_str() {
            "execute" => {
                let code = request
                    .params
                    .as_ref()
                    .and_then(|p| p.get("code"))
                    .and_then(|c| c.as_str())
                    .unwrap_or("");

                let tool = request
                    .params
                    .as_ref()
                    .and_then(|p| p.get("tool"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("default");

                // Check if approval is required
                let needs_approval = if let Some(tool_policy) = self.policy.get_tool(tool) {
                    tool_policy.requires_approval
                } else {
                    false
                };

                if needs_approval {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32001,
                            message: format!(
                                "Tool '{}' requires approval. Use 'approve' method first.",
                                tool
                            ),
                        }),
                        id: request.id,
                    };
                }

                let result = {
                    let aegis = self.aegis.lock().unwrap();
                    let aegis = aegis.with_policy(&self.policy, tool);
                    aegis.execute(code)
                };

                match result {
                    Ok(r) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::json!({ "value": format!("{:?}", r) })),
                        error: None,
                        id: request.id,
                    },
                    Err(e) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32000,
                            message: e,
                        }),
                        id: request.id,
                    },
                }
            }
            "list_policies" => {
                let policies: Vec<String> = self.policy.tools.keys().cloned().collect();
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(serde_json::json!({ "policies": policies })),
                    error: None,
                    id: request.id,
                }
            }
            "approve" => {
                // Approve a tool for execution (simple approval for now)
                let tool = request
                    .params
                    .as_ref()
                    .and_then(|p| p.get("tool"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                // For now, just acknowledge - in production would check auth
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(
                        serde_json::json!({ "approved": tool, "message": "Tool approved for execution" }),
                    ),
                    error: None,
                    id: request.id,
                }
            }
            _ => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: format!("Method not found: {}", request.method),
                }),
                id: request.id,
            },
        };

        response
    }
}
