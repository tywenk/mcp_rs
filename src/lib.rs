use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;

// Core protocol types
const JSONRPC_VERSION: &str = "2.0";
const PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    jsonrpc: String,
    id: RequestId,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    jsonrpc: String,
    id: RequestId,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<ErrorResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum RequestId {
    String(String),
    Number(i64),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

// Server implementation
pub struct Server {
    capabilities: ServerCapabilities,
    implementation: Implementation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    logging: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompts: Option<PromptsCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resources: Option<ResourcesCapability>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<ToolsCapability>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Implementation {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PromptsCapability {
    list_changed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResourcesCapability {
    subscribe: bool,
    list_changed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolsCapability {
    list_changed: bool,
}

impl Server {
    pub fn new(name: &str, version: &str) -> Self {
        Server {
            capabilities: ServerCapabilities {
                logging: Some(Value::Object(serde_json::Map::new())),
                prompts: Some(PromptsCapability {
                    list_changed: false,
                }),
                resources: Some(ResourcesCapability {
                    subscribe: false,
                    list_changed: false,
                }),
                tools: Some(ToolsCapability {
                    list_changed: false,
                }),
            },
            implementation: Implementation {
                name: name.to_string(),
                version: version.to_string(),
            },
        }
    }

    pub fn handle_message(&self, message: &str) -> Result<Option<String>, Box<dyn Error>> {
        let parsed: Value = serde_json::from_str(message)?;

        // Handle request vs notification
        if parsed.get("id").is_some() {
            self.handle_request(message)
        } else {
            self.handle_notification(message)?;
            Ok(None)
        }
    }

    fn handle_request(&self, message: &str) -> Result<Option<String>, Box<dyn Error>> {
        let request: Request = serde_json::from_str(message)?;

        match request.method.as_str() {
            "initialize" => {
                let response = Response {
                    jsonrpc: JSONRPC_VERSION.to_string(),
                    id: request.id,
                    result: Some(serde_json::json!({
                        "protocolVersion": PROTOCOL_VERSION,
                        "capabilities": self.capabilities,
                        "serverInfo": self.implementation,
                    })),
                    error: None,
                };
                Ok(Some(serde_json::to_string(&response)?))
            }
            "ping" => {
                let response = Response {
                    jsonrpc: JSONRPC_VERSION.to_string(),
                    id: request.id,
                    result: Some(Value::Object(serde_json::Map::new())),
                    error: None,
                };
                Ok(Some(serde_json::to_string(&response)?))
            }
            _ => {
                let error = ErrorResponse {
                    code: -32601, // Method not found
                    message: "Method not found".to_string(),
                    data: None,
                };
                let response = Response {
                    jsonrpc: JSONRPC_VERSION.to_string(),
                    id: request.id,
                    result: None,
                    error: Some(error),
                };
                Ok(Some(serde_json::to_string(&response)?))
            }
        }
    }

    fn handle_notification(&self, message: &str) -> Result<(), Box<dyn Error>> {
        let notification: Notification = serde_json::from_str(message)?;

        match notification.method.as_str() {
            "notifications/initialized" => Ok(()),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialization() {
        let server = Server::new("test-server", "1.0.0");

        // Test initialize request
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        });

        let response = server
            .handle_message(&init_request.to_string())
            .unwrap()
            .unwrap();
        let response_value: Value = serde_json::from_str(&response).unwrap();

        assert_eq!(response_value["result"]["protocolVersion"], "2024-11-05");
        assert!(response_value["result"]["capabilities"].is_object());
    }

    #[test]
    fn test_ping() {
        let server = Server::new("test-server", "1.0.0");

        let ping_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "ping"
        });

        let response = server
            .handle_message(&ping_request.to_string())
            .unwrap()
            .unwrap();
        let response_value: Value = serde_json::from_str(&response).unwrap();

        assert!(response_value["result"].is_object());
        assert!(response_value["error"].is_null());
    }
}
