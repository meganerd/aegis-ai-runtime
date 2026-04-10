use crate::{Aegis, ExecutionStateManager, Policy};
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

const MAX_REQUEST_SIZE: usize = 1_000_000;
const ALLOWED_METHODS: [&str; 5] = [
    "execute",
    "approve",
    "list_executions",
    "get_execution",
    "validate",
];

pub struct McpServer {
    aegis: Arc<Mutex<Aegis>>,
    policy: Arc<Policy>,
    exec_manager: ExecutionStateManager,
}

impl McpServer {
    fn validate_method(&self, method: &str) -> Result<(), String> {
        if !ALLOWED_METHODS.contains(&method) {
            return Err(format!("Unknown method: {}", method));
        }
        Ok(())
    }

    fn validate_request_size(&self, body: &str) -> Result<(), String> {
        if body.len() > MAX_REQUEST_SIZE {
            return Err("Request too large".to_string());
        }
        Ok(())
    }
}

impl McpServer {
    pub fn new(aegis: Aegis, policy: Policy) -> Self {
        Self {
            aegis: Arc::new(Mutex::new(aegis)),
            policy: Arc::new(policy),
            exec_manager: ExecutionStateManager::new(),
        }
    }

    pub fn handle_request(&self, body: &str) -> JsonRpcResponse {
        if let Err(e) = self.validate_request_size(body) {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32600,
                    message: e,
                }),
                id: None,
            };
        }

        let request: JsonRpcRequest = match serde_json::from_str(body) {
            Ok(r) => r,
            Err(e) => {
                return JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: "Invalid JSON".to_string(),
                    }),
                    id: None,
                };
            }
        };

        if let Err(e) = self.validate_method(&request.method) {
            return JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                result: None,
                error: Some(JsonRpcError {
                    code: -32601,
                    message: e,
                }),
                id: request.id,
            };
        }

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
                    // Create execution record
                    let exec_id = self.exec_manager.create(code, tool);
                    self.exec_manager.start(&exec_id).ok();

                    let aegis = self.aegis.lock().unwrap();
                    let aegis = aegis.with_policy(&self.policy, tool);
                    aegis.execute(code)
                };

                match result {
                    Ok(r) => {
                        // Record completion - need exec_id from above, this is simplified
                        JsonRpcResponse {
                            jsonrpc: "2.0".to_string(),
                            result: Some(serde_json::json!({ "value": format!("{:?}", r) })),
                            error: None,
                            id: request.id,
                        }
                    }
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
            "list_executions" => {
                let executions: Vec<serde_json::Value> = self
                    .exec_manager
                    .list()
                    .iter()
                    .map(|e| {
                        serde_json::json!({
                            "id": e.id,
                            "tool": e.tool,
                            "state": e.state,
                            "created_at": e.created_at
                        })
                    })
                    .collect();
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    result: Some(serde_json::json!({ "executions": executions })),
                    error: None,
                    id: request.id,
                }
            }
            "pause_execution" => {
                let id = request
                    .params
                    .as_ref()
                    .and_then(|p| p.get("id"))
                    .and_then(|i| i.as_str())
                    .unwrap_or("");

                match self.exec_manager.pause(id) {
                    Ok(_) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::json!({ "paused": id })),
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
            "resume_execution" => {
                let id = request
                    .params
                    .as_ref()
                    .and_then(|p| p.get("id"))
                    .and_then(|i| i.as_str())
                    .unwrap_or("");

                match self.exec_manager.resume(id) {
                    Ok(_) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: Some(serde_json::json!({ "resumed": id })),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_method() {
        let aegis = Aegis::new();
        let policy_yaml = std::fs::read_to_string("policy.yaml").unwrap();
        let policy = Policy::from_yaml(&policy_yaml).unwrap();
        let server = McpServer::new(aegis, policy);

        assert!(server.validate_method("execute").is_ok());
        assert!(server.validate_method("approve").is_ok());
        assert!(server.validate_method("unknown").is_err());
    }

    #[test]
    fn test_validate_request_size() {
        let aegis = Aegis::new();
        let policy_yaml = std::fs::read_to_string("policy.yaml").unwrap();
        let policy = Policy::from_yaml(&policy_yaml).unwrap();
        let server = McpServer::new(aegis, policy);

        assert!(server.validate_request_size("{}").is_ok());
        assert!(server.validate_request_size(&"x".repeat(1_000_001)).is_err());
    }
}
