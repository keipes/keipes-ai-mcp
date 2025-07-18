pub mod handlers;
pub mod types;

use axum::{
    extract::{Query, Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use handlers::{PromptHandler, ResourceHandler, ToolHandler};
use serde::Deserialize;
use serde_json;
use std::collections::HashMap;
use tokio::net::TcpListener;
use tracing::{error, info};
use types::{McpCapabilities, ServerConfig, ServerDetails, ServerInfo};
use uuid::Uuid;


#[derive(Clone)]
pub struct McpServer {
    server_info: ServerInfo,
    config: ServerConfig,
    prompt_handler: PromptHandler,
    tool_handler: ToolHandler,
    resource_handler: ResourceHandler,
    sessions: std::sync::Arc<tokio::sync::RwLock<HashMap<String, SessionData>>>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
struct SessionData {
    id: String,
    created_at: std::time::Instant,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SessionParams {
    session_id: Option<String>,
}

impl McpServer {
    pub async fn new(config: ServerConfig) -> Self {
        let prompt_handler = PromptHandler::new();
        let tool_handler = ToolHandler::new().await;
        let resource_handler = ResourceHandler::new();
        Self {
            server_info: ServerInfo {
                protocol_version: "2025-06-18".to_string(),
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
            sessions: std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    pub async fn start(&self) -> Result<(), String> {
        println!(
            "Starting MCP Server on {}:{}",
            self.config.bind_address, self.config.port
        );

        let server = self.clone();
        let app = Router::new()
            .route("/mcp", post(Self::handle_post).get(Self::handle_get))
            .route("/health", get(|| async { "OK" }))
            // default 404 status for unmatched routes or methods
            .fallback(get(|| async { StatusCode::NOT_FOUND }))
            // fallback for post
            .fallback(post(|| async { StatusCode::NOT_FOUND }))
            .layer(middleware::from_fn(Self::log_requests))
            .with_state(server);

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

    // Origin validation for both local and remote scenarios
    fn validate_origin(headers: &HeaderMap) -> Result<(), StatusCode> {
        let allowed_origins = [
            // Local development
            "http://localhost:3000",
            "http://localhost:8080",
            "http://127.0.0.1:3000",
            "http://127.0.0.1:8080",
            // Anthropic remote-mcp client
            "https://claude.ai",
            "https://anthropic.com",
            "https://api.anthropic.com",
            // Allow null origin for local tools
            "null",
        ];

        if let Some(origin) = headers.get("Origin") {
            let origin_str = origin.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
            if !allowed_origins.contains(&origin_str) {
                return Err(StatusCode::FORBIDDEN);
            }
        }
        Ok(())
    }

    // Validate MCP protocol version header
    fn validate_protocol_version(headers: &HeaderMap) -> Result<(), StatusCode> {
        if let Some(version) = headers.get("MCP-Protocol-Version") {
            let version_str = version.to_str().map_err(|_| StatusCode::BAD_REQUEST)?;
            if version_str != "2025-06-18" && version_str != "2024-11-05" {
                return Err(StatusCode::BAD_REQUEST);
            }
        }
        Ok(())
    }

    // Request logging middleware
    async fn log_requests(req: Request, next: Next) -> Response {
        let method = req.method().clone();
        let uri = req.uri().clone();
        let headers = req.headers().clone();
        let origin = headers.get("Origin").and_then(|h| h.to_str().ok()).unwrap_or("none");
        
        info!("Request - {} {} origin: {}", method, uri, origin);
        
        let response = next.run(req).await;
        
        info!("Response - {} {} status: {}", method, uri, response.status());
        
        response
    }

    // Handle POST requests (client to server messages)
    async fn handle_post(
        State(server): State<McpServer>,
        headers: HeaderMap,
        Json(payload): Json<serde_json::Value>,
    ) -> Result<Response, StatusCode> {
        let method = payload.get("method").and_then(|m| m.as_str()).unwrap_or("unknown");
        let id = payload.get("id");
        let origin = headers.get("Origin").and_then(|h| h.to_str().ok()).unwrap_or("none");
        
        info!("Received POST request: method={}, id={:?}, origin={}", method, id, origin);

        // Validate origin and protocol version
        if let Err(status) = Self::validate_origin(&headers) {
            error!("Origin validation failed for {}: {:?}", origin, status);
            return Err(status);
        }
        
        if let Err(status) = Self::validate_protocol_version(&headers) {
            let version = headers.get("MCP-Protocol-Version").and_then(|h| h.to_str().ok()).unwrap_or("none");
            error!("Protocol version validation failed for {}: {:?}", version, status);
            return Err(status);
        }

        // Check Accept header
        let accept = headers.get("Accept").and_then(|h| h.to_str().ok()).unwrap_or("");
        let supports_sse = accept.contains("text/event-stream");
        let supports_json = accept.contains("application/json");

        if !supports_sse && !supports_json {
            error!("Invalid Accept header for method {}: {}", method, accept);
            return Err(StatusCode::BAD_REQUEST);
        }

        info!("Accept header valid: supports_sse={}, supports_json={}", supports_sse, supports_json);

        let id = payload.get("id").unwrap_or(&serde_json::Value::Null);

        // Handle session management
        let session_id = headers
            .get("Mcp-Session-Id")
            .and_then(|h| h.to_str().ok())
            .map(|s| s.to_string());

        info!("Processing method: {}, session_id: {:?}", method, session_id);

        match method {
            "initialize" => {
                info!("Handling initialize request");
                let mut response_headers = HeaderMap::new();
                
                // Create new session for initialize
                let new_session_id = Uuid::new_v4().to_string();
                let session_data = SessionData {
                    id: new_session_id.clone(),
                    created_at: std::time::Instant::now(),
                };
                
                server.sessions.write().await.insert(new_session_id.clone(), session_data);
                response_headers.insert(
                    "Mcp-Session-Id",
                    HeaderValue::from_str(&new_session_id).unwrap(),
                );

                info!("Created new session: {}", new_session_id);

                let result = serde_json::json!(server.server_info);
                let response = Self::jsonrpc_result_value(id, result);
                
                let mut resp = Json(response).into_response();
                resp.headers_mut().extend(response_headers);
                Ok(resp)
            }
            "tools/list" => {
                info!("Handling tools/list request");
                let tools_result = server.tool_handler.list_tools(None).await;
                let response = Self::jsonrpc_result_value(id, serde_json::to_value(tools_result).unwrap());
                Ok(Json(response).into_response())
            }
            "tools/call" => {
                info!("Handling tools/call request");
                if let Some(params) = payload.get("params") {
                    if let Ok(call_request) =
                        serde_json::from_value::<rmcp::model::CallToolRequestParam>(params.clone())
                    {
                        info!("Calling tool: {}", call_request.name);
                        match server.tool_handler.call_tool(call_request).await {
                            Ok(result) => {
                                info!("Tool call successful");
                                let response = Self::jsonrpc_result_value(id, serde_json::to_value(result).unwrap());
                                Ok(Json(response).into_response())
                            }
                            Err(error) => {
                                error!("Tool call failed: {:?}", error);
                                let response = Self::jsonrpc_error_with_data_value(id, error);
                                Ok(Json(response).into_response())
                            }
                        }
                    } else {
                        error!("Invalid tool call parameters");
                        let response = Self::jsonrpc_error_value(id, -32602, "Invalid request parameters");
                        Ok(Json(response).into_response())
                    }
                } else {
                    error!("Missing params in tool call");
                    let response = Self::jsonrpc_error_value(id, -32602, "Invalid params");
                    Ok(Json(response).into_response())
                }
            }
            "resources/list" => {
                info!("Handling resources/list request");
                let params = payload
                    .get("params")
                    .and_then(|p| serde_json::from_value(p.clone()).ok());
                let resources_result = server.resource_handler.list_resources(params).await;
                let response = Self::jsonrpc_result_value(id, serde_json::to_value(resources_result).unwrap());
                Ok(Json(response).into_response())
            }
            "resources/read" => {
                info!("Handling resources/read request");
                if let Some(params) = payload.get("params") {
                    if let Ok(read_request) = serde_json::from_value::<
                        rmcp::model::ReadResourceRequestParam,
                    >(params.clone())
                    {
                        info!("Reading resource: {}", read_request.uri);
                        match server.resource_handler.read_resource(read_request).await {
                            Ok(result) => {
                                info!("Resource read successful");
                                let response = Self::jsonrpc_result_value(id, serde_json::to_value(result).unwrap());
                                Ok(Json(response).into_response())
                            }
                            Err(error) => {
                                error!("Resource read failed: {:?}", error);
                                let response = Self::jsonrpc_error_with_data_value(id, error);
                                Ok(Json(response).into_response())
                            }
                        }
                    } else {
                        error!("Invalid resource read parameters");
                        let response = Self::jsonrpc_error_value(id, -32602, "Invalid request parameters");
                        Ok(Json(response).into_response())
                    }
                } else {
                    error!("Missing params in resource read");
                    let response = Self::jsonrpc_error_value(id, -32602, "Invalid params");
                    Ok(Json(response).into_response())
                }
            }
            "resources/templates/list" => {
                info!("Handling resources/templates/list request");
                let params = payload
                    .get("params")
                    .and_then(|p| serde_json::from_value(p.clone()).ok());
                let templates_result = server.resource_handler.list_resource_templates(params).await;
                let response = Self::jsonrpc_result_value(id, serde_json::to_value(templates_result).unwrap());
                Ok(Json(response).into_response())
            }
            "prompts/list" => {
                info!("Handling prompts/list request");
                let params = payload
                    .get("params")
                    .and_then(|p| serde_json::from_value(p.clone()).ok());
                let prompts_result = server.prompt_handler.list_prompts(params).await;
                let response = Self::jsonrpc_result_value(id, serde_json::to_value(prompts_result).unwrap());
                Ok(Json(response).into_response())
            }
            "prompts/get" => {
                info!("Handling prompts/get request");
                if let Some(params) = payload.get("params") {
                    if let Ok(get_request) =
                        serde_json::from_value::<rmcp::model::GetPromptRequestParam>(params.clone())
                    {
                        info!("Getting prompt: {}", get_request.name);
                        match server.prompt_handler.get_prompt(get_request).await {
                            Ok(result) => {
                                info!("Prompt get successful");
                                let response = Self::jsonrpc_result_value(id, serde_json::to_value(result).unwrap());
                                Ok(Json(response).into_response())
                            }
                            Err(error) => {
                                error!("Prompt get failed: {:?}", error);
                                let response = Self::jsonrpc_error_with_data_value(id, error);
                                Ok(Json(response).into_response())
                            }
                        }
                    } else {
                        error!("Invalid prompt get parameters");
                        let response = Self::jsonrpc_error_value(id, -32602, "Invalid request parameters");
                        Ok(Json(response).into_response())
                    }
                } else {
                    error!("Missing params in prompt get");
                    let response = Self::jsonrpc_error_value(id, -32602, "Invalid params");
                    Ok(Json(response).into_response())
                }
            }
            _ => {
                // Validate session for non-initialize requests
                if let Some(session_id) = &session_id {
                    let sessions = server.sessions.read().await;
                    if !sessions.contains_key(session_id) {
                        error!("Session not found for method {}: {}", method, session_id);
                        return Err(StatusCode::NOT_FOUND);
                    }
                } else if method != "initialize" {
                    error!("Missing session ID for method: {}", method);
                    return Err(StatusCode::BAD_REQUEST);
                }

                error!("Unknown method: {}", method);
                let response = Self::jsonrpc_error_value(id, -32601, "Method not found");
                Ok(Json(response).into_response())
            }
        }
    }

    // Handle GET requests (SSE streams)
    async fn handle_get(
        State(_server): State<McpServer>,
        headers: HeaderMap,
        _query: Query<SessionParams>,
    ) -> Result<Response, StatusCode> {
        let origin = headers.get("Origin").and_then(|h| h.to_str().ok()).unwrap_or("none");
        info!("Received GET request for SSE stream, origin={}", origin);

        // Validate origin and protocol version
        if let Err(status) = Self::validate_origin(&headers) {
            error!("Origin validation failed for GET request from {}: {:?}", origin, status);
            return Err(status);
        }
        
        if let Err(status) = Self::validate_protocol_version(&headers) {
            let version = headers.get("MCP-Protocol-Version").and_then(|h| h.to_str().ok()).unwrap_or("none");
            error!("Protocol version validation failed for GET request with version {}: {:?}", version, status);
            return Err(status);
        }

        // Check Accept header for SSE support
        let accept = headers.get("Accept").and_then(|h| h.to_str().ok()).unwrap_or("");
        if !accept.contains("text/event-stream") {
            error!("GET request without SSE support in Accept header: {}", accept);
            return Err(StatusCode::METHOD_NOT_ALLOWED);
        }

        // For now, return method not allowed as we don't have server-initiated messages
        // In a full implementation, this would open an SSE stream
        info!("SSE stream not implemented, returning method not allowed");
        Err(StatusCode::METHOD_NOT_ALLOWED)
    }

}

// JSON-RPC helper methods
impl McpServer {
    fn jsonrpc_result_value(
        id: &serde_json::Value,
        result: serde_json::Value,
    ) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": result
        })
    }

    fn jsonrpc_error_value(id: &serde_json::Value, code: i32, message: &str) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": code,
                "message": message
            }
        })
    }

    fn jsonrpc_error_with_data_value(
        id: &serde_json::Value,
        error: rmcp::model::ErrorData,
    ) -> serde_json::Value {
        serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": error
        })
    }
}
