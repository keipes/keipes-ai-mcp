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

// JSON-RPC helper methods
fn jsonrpc_result(id: &serde_json::Value, result: serde_json::Value) -> Json<serde_json::Value> {
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

fn jsonrpc_error_with_data(id: &serde_json::Value, error: rmcp::model::ErrorData) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": error
    }))
}

async fn handle_mcp_request(Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = payload.get("id").unwrap_or(&serde_json::Value::Null);
    
    let tool_handler = ToolHandler::new();
    let resource_handler = ResourceHandler::new();
    let prompt_handler = PromptHandler::new();
    
    match method {
        "initialize" => {
            let result = serde_json::json!({
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
            });
            jsonrpc_result(id, result)
        },
        "tools/list" => {
            let tools_result = tool_handler.list_tools(None).await;
            jsonrpc_result(id, serde_json::to_value(tools_result).unwrap())
        },
        "tools/call" => {
            if let Some(params) = payload.get("params") {
                if let Ok(call_request) = serde_json::from_value::<rmcp::model::CallToolRequestParam>(params.clone()) {
                    match tool_handler.call_tool(call_request).await {
                        Ok(result) => jsonrpc_result(id, serde_json::to_value(result).unwrap()),
                        Err(error) => jsonrpc_error_with_data(id, error)
                    }
                } else {
                    jsonrpc_error(id, -32602, "Invalid request parameters")
                }
            } else {
                jsonrpc_error(id, -32602, "Invalid params")
            }
        },
        "resources/list" => {
            let params = payload.get("params").and_then(|p| serde_json::from_value(p.clone()).ok());
            let resources_result = resource_handler.list_resources(params).await;
            jsonrpc_result(id, serde_json::to_value(resources_result).unwrap())
        },
        "resources/read" => {
            if let Some(params) = payload.get("params") {
                if let Ok(read_request) = serde_json::from_value::<rmcp::model::ReadResourceRequestParam>(params.clone()) {
                    match resource_handler.read_resource(read_request).await {
                        Ok(result) => jsonrpc_result(id, serde_json::to_value(result).unwrap()),
                        Err(error) => jsonrpc_error_with_data(id, error)
                    }
                } else {
                    jsonrpc_error(id, -32602, "Invalid request parameters")
                }
            } else {
                jsonrpc_error(id, -32602, "Invalid params")
            }
        },
        "prompts/list" => {
            let params = payload.get("params").and_then(|p| serde_json::from_value(p.clone()).ok());
            let prompts_result = prompt_handler.list_prompts(params).await;
            jsonrpc_result(id, serde_json::to_value(prompts_result).unwrap())
        },
        "prompts/get" => {
            if let Some(params) = payload.get("params") {
                if let Ok(get_request) = serde_json::from_value::<rmcp::model::GetPromptRequestParam>(params.clone()) {
                    match prompt_handler.get_prompt(get_request).await {
                        Ok(result) => jsonrpc_result(id, serde_json::to_value(result).unwrap()),
                        Err(error) => jsonrpc_error_with_data(id, error)
                    }
                } else {
                    jsonrpc_error(id, -32602, "Invalid request parameters")
                }
            } else {
                jsonrpc_error(id, -32602, "Invalid params")
            }
        },
        _ => {
            jsonrpc_error(id, -32601, "Method not found")
        }
    }
}
