pub mod handlers;
pub mod types;

use handlers::{PromptHandler, ToolHandler, ResourceHandler};
use types::{ServerConfig, ServerInfo, McpCapabilities, ServerDetails};
use axum::{Json, routing::{get, post}, Router};
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct McpServer {
    server_info: ServerInfo,
    config: ServerConfig,
    prompt_handler: PromptHandler,
    tool_handler: ToolHandler,
    resource_handler: ResourceHandler,
}

impl McpServer {
    pub fn new(config: ServerConfig) -> Self {
        let prompt_handler = PromptHandler::new();
        let tool_handler = ToolHandler::new();
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
        println!("Starting MCP Server on {}:{}", self.config.bind_address, self.config.port);
        
        let server = self.clone();
        let app = Router::new()
            .route("/mcp", post({
                move |payload| async move { server.handle_mcp_request(payload).await }
            }))
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

    async fn handle_mcp_request(&self, Json(payload): Json<serde_json::Value>) -> Json<serde_json::Value> {
        let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = payload.get("id").unwrap_or(&serde_json::Value::Null);
        
        match method {
            "initialize" => {
                let result = serde_json::json!(self.server_info);
                Self::jsonrpc_result(id, result)
            },
            "tools/list" => {
                let tools_result = self.tool_handler.list_tools(None).await;
                Self::jsonrpc_result(id, serde_json::to_value(tools_result).unwrap())
            },
            "tools/call" => {
                if let Some(params) = payload.get("params") {
                    if let Ok(call_request) = serde_json::from_value::<rmcp::model::CallToolRequestParam>(params.clone()) {
                        match self.tool_handler.call_tool(call_request).await {
                            Ok(result) => Self::jsonrpc_result(id, serde_json::to_value(result).unwrap()),
                            Err(error) => Self::jsonrpc_error_with_data(id, error)
                        }
                    } else {
                        Self::jsonrpc_error(id, -32602, "Invalid request parameters")
                    }
                } else {
                    Self::jsonrpc_error(id, -32602, "Invalid params")
                }
            },
            "resources/list" => {
                let params = payload.get("params").and_then(|p| serde_json::from_value(p.clone()).ok());
                let resources_result = self.resource_handler.list_resources(params).await;
                Self::jsonrpc_result(id, serde_json::to_value(resources_result).unwrap())
            },
            "resources/read" => {
                if let Some(params) = payload.get("params") {
                    if let Ok(read_request) = serde_json::from_value::<rmcp::model::ReadResourceRequestParam>(params.clone()) {
                        match self.resource_handler.read_resource(read_request).await {
                            Ok(result) => Self::jsonrpc_result(id, serde_json::to_value(result).unwrap()),
                            Err(error) => Self::jsonrpc_error_with_data(id, error)
                        }
                    } else {
                        Self::jsonrpc_error(id, -32602, "Invalid request parameters")
                    }
                } else {
                    Self::jsonrpc_error(id, -32602, "Invalid params")
                }
            },
            "resources/templates/list" => {
                let params = payload.get("params").and_then(|p| serde_json::from_value(p.clone()).ok());
                let templates_result = self.resource_handler.list_resource_templates(params).await;
                Self::jsonrpc_result(id, serde_json::to_value(templates_result).unwrap())
            },
            "prompts/list" => {
                let params = payload.get("params").and_then(|p| serde_json::from_value(p.clone()).ok());
                let prompts_result = self.prompt_handler.list_prompts(params).await;
                Self::jsonrpc_result(id, serde_json::to_value(prompts_result).unwrap())
            },
            "prompts/get" => {
                if let Some(params) = payload.get("params") {
                    if let Ok(get_request) = serde_json::from_value::<rmcp::model::GetPromptRequestParam>(params.clone()) {
                        match self.prompt_handler.get_prompt(get_request).await {
                            Ok(result) => Self::jsonrpc_result(id, serde_json::to_value(result).unwrap()),
                            Err(error) => Self::jsonrpc_error_with_data(id, error)
                        }
                    } else {
                        Self::jsonrpc_error(id, -32602, "Invalid request parameters")
                    }
                } else {
                    Self::jsonrpc_error(id, -32602, "Invalid params")
                }
            },
            _ => {
                Self::jsonrpc_error(id, -32601, "Method not found")
            }
        }
    }
}

// JSON-RPC helper methods
impl McpServer {
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
}
