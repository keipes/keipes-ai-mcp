pub mod handlers;
pub mod types;

use handlers::{PromptHandler, ToolHandler, ResourceHandler};
use types::{ServerConfig, ServerInfo, ServerCapabilities};
use std::sync::Arc;
use axum::{Json, routing::{get, post}, Router};
use tokio::net::TcpListener;

pub struct McpServer {
    server_info: ServerInfo,
    config: ServerConfig,
    prompt_handler: Arc<PromptHandler>,
    tool_handler: Arc<ToolHandler>,
    resource_handler: Arc<ResourceHandler>,
}

impl McpServer {
    pub fn new(config: ServerConfig) -> Self {
        Self {
            server_info: ServerInfo {
                name: "keipes-ai-mcp".to_string(),
                version: "0.1.0".to_string(),
                capabilities: ServerCapabilities {
                    tools: true,
                    resources: true,
                    prompts: true,
                },
            },
            config,
            prompt_handler: Arc::new(PromptHandler::new()),
            tool_handler: Arc::new(ToolHandler::new()),
            resource_handler: Arc::new(ResourceHandler::new()),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        println!("Starting MCP Server on {}:{}", self.config.bind_address, self.config.port);
        
        let app = Router::new()
            .route("/mcp", post(handle_mcp_request))
            .route("/health", get(|| async { "OK" }));

        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = TcpListener::bind(&addr).await.map_err(|e| e.to_string())?;
        println!("MCP Server listening on {}", addr);
        
        axum::serve(listener, app).await.map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn shutdown(&self) {
        // Graceful shutdown will be handled by dropping the server
    }
}

async fn handle_mcp_request(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = payload.get("id").unwrap_or(&serde_json::Value::Null);
    
    let tool_handler = ToolHandler::new();
    
    match method {
        "initialize" => {
            Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {},
                        "resources": {},
                        "prompts": {}
                    },
                    "serverInfo": {
                        "name": "keipes-ai-mcp",
                        "version": "0.1.0"
                    }
                }
            }))
        },
        "tools/list" => {
            let tools_result = tool_handler.list_tools(None).await;
            Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": tools_result
            }))
        },
        "tools/call" => {
            if let Some(params) = payload.get("params") {
                if let Ok(call_request) = serde_json::from_value::<rmcp::model::CallToolRequestParam>(params.clone()) {
                    match tool_handler.call_tool(call_request).await {
                        Ok(result) => Json(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": result
                        })),
                        Err(error) => Json(serde_json::json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": error
                        }))
                    }
                } else {
                    Json(serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32602,
                            "message": "Invalid request parameters"
                        }
                    }))
                }
            } else {
                Json(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params"
                    }
                }))
            }
        },
        _ => {
            Json(serde_json::json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": "Method not found"
                }
            }))
        }
    }
}
