pub mod handlers;
pub mod types;

use axum::{
    routing::{get, post},
    Json, Router,
};
use handlers::{PromptHandler, ResourceHandler, ToolHandler};
use rmcp::model::*;
use serde_json;
use tokio::net::TcpListener;
use types::{McpCapabilities, ServerConfig, ServerDetails, ServerInfo};

use std::collections::HashMap;

#[derive(Clone)]
pub struct McpServer {
    server_info: ServerInfo,
    config: ServerConfig,
    prompt_handler: PromptHandler,
    tool_handler: ToolHandler,
    resource_handler: ResourceHandler,
}

impl McpServer {
    pub async fn new(config: ServerConfig) -> Self {
        let prompt_handler = PromptHandler::new();
        let tool_handler = ToolHandler::new().await;
        let resource_handler = ResourceHandler::new();
        Self {
            server_info: ServerInfo {
                protocol_version: "2024-11-05".to_string(),
                capabilities: McpCapabilities {
                    tools: tool_handler.capabilities(),
                    resources: resource_handler.capabilities(),
                    prompts: prompt_handler.capabilities(),
                },
                server_info: ServerDetails {
                    name: "keipes-ai-mcp".to_string(),
                    version: "0.1.0".to_string(),
                },
            },
            config,
            prompt_handler: prompt_handler,
            tool_handler: tool_handler,
            resource_handler: resource_handler,
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        println!(
            "Starting MCP Server on {}:{}",
            self.config.bind_address, self.config.port
        );

        let server = self.clone();
        let app = Router::new()
            .route(
                "/mcp",
                post({ move |payload| async move { server.handle_mcp_request(payload).await } }),
            )
            .route("/health", get(|| async { "OK" }));

        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = TcpListener::bind(&addr).await.map_err(|e| e.to_string())?;
        println!("MCP Server listening on {}", addr);

        axum::serve(listener, app)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn shutdown(&self) {
        // Graceful shutdown will be handled by dropping the server
    }

    async fn handle_mcp_request(
        &self,
        Json(payload): Json<serde_json::Value>,
    ) -> Json<serde_json::Value> {
        let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = payload.get("id").unwrap_or(&serde_json::Value::Null);

        match method {
            "initialize" => {
                let result = serde_json::json!(self.server_info);
                Self::jsonrpc_result(id, result)
            }
            "tools/list" => {
                let tools_result = self.tool_handler.list_tools(None).await;
                Self::jsonrpc_result(id, serde_json::to_value(tools_result).unwrap())
            }
            "tools/call" => {
                if let Some(params) = payload.get("params") {
                    if let Ok(call_request) =
                        serde_json::from_value::<rmcp::model::CallToolRequestParam>(params.clone())
                    {
                        match self.tool_handler.call_tool(call_request).await {
                            Ok(result) => {
                                Self::jsonrpc_result(id, serde_json::to_value(result).unwrap())
                            }
                            Err(error) => Self::jsonrpc_error_with_data(id, error),
                        }
                    } else {
                        Self::jsonrpc_error(id, -32602, "Invalid request parameters")
                    }
                } else {
                    Self::jsonrpc_error(id, -32602, "Invalid params")
                }
            }
            "resources/list" => {
                let params = payload
                    .get("params")
                    .and_then(|p| serde_json::from_value(p.clone()).ok());
                let resources_result = self.resource_handler.list_resources(params).await;
                Self::jsonrpc_result(id, serde_json::to_value(resources_result).unwrap())
            }
            "resources/read" => {
                if let Some(params) = payload.get("params") {
                    if let Ok(read_request) = serde_json::from_value::<
                        rmcp::model::ReadResourceRequestParam,
                    >(params.clone())
                    {
                        match self.resource_handler.read_resource(read_request).await {
                            Ok(result) => {
                                Self::jsonrpc_result(id, serde_json::to_value(result).unwrap())
                            }
                            Err(error) => Self::jsonrpc_error_with_data(id, error),
                        }
                    } else {
                        Self::jsonrpc_error(id, -32602, "Invalid request parameters")
                    }
                } else {
                    Self::jsonrpc_error(id, -32602, "Invalid params")
                }
            }
            "resources/templates/list" => {
                let params = payload
                    .get("params")
                    .and_then(|p| serde_json::from_value(p.clone()).ok());
                let templates_result = self.resource_handler.list_resource_templates(params).await;
                Self::jsonrpc_result(id, serde_json::to_value(templates_result).unwrap())
            }
            "prompts/list" => {
                let params = payload
                    .get("params")
                    .and_then(|p| serde_json::from_value(p.clone()).ok());
                let prompts_result = self.prompt_handler.list_prompts(params).await;
                Self::jsonrpc_result(id, serde_json::to_value(prompts_result).unwrap())
            }
            "prompts/get" => {
                if let Some(params) = payload.get("params") {
                    if let Ok(get_request) =
                        serde_json::from_value::<rmcp::model::GetPromptRequestParam>(params.clone())
                    {
                        match self.prompt_handler.get_prompt(get_request).await {
                            Ok(result) => {
                                Self::jsonrpc_result(id, serde_json::to_value(result).unwrap())
                            }
                            Err(error) => Self::jsonrpc_error_with_data(id, error),
                        }
                    } else {
                        Self::jsonrpc_error(id, -32602, "Invalid request parameters")
                    }
                } else {
                    Self::jsonrpc_error(id, -32602, "Invalid params")
                }
            }
            _ => Self::jsonrpc_error(id, -32601, "Method not found"),
        }
    }
}

// JSON-RPC helper methods
impl McpServer {
    fn jsonrpc_result(
        id: &serde_json::Value,
        result: serde_json::Value,
    ) -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        }))
    }

    fn jsonrpc_error(id: &serde_json::Value, code: i32, message: &str) -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message
            }
        }))
    }

    fn jsonrpc_error_with_data(
        id: &serde_json::Value,
        error: rmcp::model::ErrorData,
    ) -> Json<serde_json::Value> {
        Json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": error
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let config = ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 8000,
            sse_path: "/sse".to_string(),
            post_path: "/post".to_string(),
        };

        let server = McpServer::new(config.clone()).await;
        assert_eq!(server.config.bind_address, "127.0.0.1");
        assert_eq!(server.config.port, 8000);
        assert_eq!(server.server_info.protocol_version, "2024-11-05");
        assert_eq!(server.server_info.server_info.name, "keipes-ai-mcp");
        assert_eq!(server.server_info.server_info.version, "0.1.0");
    }

    #[tokio::test]
    async fn test_mcp_server_clone() {
        let config = ServerConfig {
            bind_address: "0.0.0.0".to_string(),
            port: 9000,
            sse_path: "/events".to_string(),
            post_path: "/messages".to_string(),
        };

        let server = McpServer::new(config).await;
        let cloned = server.clone();
        assert_eq!(server.config.bind_address, cloned.config.bind_address);
        assert_eq!(server.config.port, cloned.config.port);
    }

    #[test]
    fn test_jsonrpc_result() {
        let id = serde_json::Value::Number(serde_json::Number::from(1));
        let result = serde_json::json!({"success": true});

        let response = McpServer::jsonrpc_result(&id, result);
        let json = response.0;

        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert_eq!(json["result"]["success"], true);
    }

    #[test]
    fn test_jsonrpc_error() {
        let id = serde_json::Value::Number(serde_json::Number::from(2));

        let response = McpServer::jsonrpc_error(&id, -32601, "Method not found");
        let json = response.0;

        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 2);
        assert_eq!(json["error"]["code"], -32601);
        assert_eq!(json["error"]["message"], "Method not found");
    }

    #[test]
    fn test_jsonrpc_error_with_data() {
        let id = serde_json::Value::Number(serde_json::Number::from(3));
        let error = rmcp::model::ErrorData {
            code: rmcp::model::ErrorCode(-32602),
            message: "Invalid params".into(),
            data: None,
        };

        let response = McpServer::jsonrpc_error_with_data(&id, error);
        let json = response.0;

        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 3);
        assert!(json["error"].is_object());
    }

    #[tokio::test]
    async fn test_handle_mcp_request_initialize() {
        let config = ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 8000,
            sse_path: "/sse".to_string(),
            post_path: "/post".to_string(),
        };

        let server = McpServer::new(config).await;
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "id": 1
        });

        let response = server.handle_mcp_request(Json(request)).await;
        let json = response.0;

        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert!(json["result"]["protocolVersion"].is_string());
    }

    #[tokio::test]
    async fn test_handle_mcp_request_unknown_method() {
        let config = ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 8000,
            sse_path: "/sse".to_string(),
            post_path: "/post".to_string(),
        };

        let server = McpServer::new(config).await;
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "unknown_method",
            "id": 1
        });

        let response = server.handle_mcp_request(Json(request)).await;
        let json = response.0;

        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert_eq!(json["error"]["code"], -32601);
        assert_eq!(json["error"]["message"], "Method not found");
    }

    #[tokio::test]
    async fn test_handle_mcp_request_tools_list() {
        let config = ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 8000,
            sse_path: "/sse".to_string(),
            post_path: "/post".to_string(),
        };

        let server = McpServer::new(config).await;
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": 1
        });

        let response = server.handle_mcp_request(Json(request)).await;
        let json = response.0;

        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["id"], 1);
        assert!(json["result"]["tools"].is_array());
    }

    #[tokio::test]
    async fn test_server_shutdown() {
        let config = ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 8000,
            sse_path: "/sse".to_string(),
            post_path: "/post".to_string(),
        };

        let server = McpServer::new(config).await;
        // Shutdown should not panic
        server.shutdown();
    }
}
