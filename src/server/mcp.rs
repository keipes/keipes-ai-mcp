//! Nexus MCP Server
//!
//! A high-performance Model Context Protocol (MCP) server implementation
//! providing advanced tool routing and capability management.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{future::Future, sync::Arc, time::Duration};
use tokio::sync::Mutex;

use rmcp::{
    handler::server::tool::{Parameters, ToolRouter},
    model::*,
    schemars::JsonSchema,
    tool, tool_handler, tool_router,
    transport::{streamable_http_server::StreamableHttpService, StreamableHttpServerConfig},
    ServerHandler,
};

use crate::tools;

pub fn create_nexus_service() -> StreamableHttpService<NexusServer> {
    // let config: StreamableHttpServerConfig = StreamableHttpServerConfig {
    //     stateful_mode: false,
    //     sse_keep_alive: None,
    // };
    // let local_session_manager = LocalSessionManager {
    //     session_config: config.clone(),
    //     sessions: Default::default(),
    // };
    // let local_session_managers = Arc::new(local_session_manager);
    // config.stateful_mode = true;
    StreamableHttpService::new(
        || Ok(NexusServer::new()),
        Default::default(),
        StreamableHttpServerConfig {
            stateful_mode: true,
            sse_keep_alive: Some(Duration::from_secs(15)),
        },
    )
}

#[derive(Clone)]
pub struct NexusServer {
    counter: Arc<Mutex<i64>>,
    tool_router: ToolRouter<NexusServer>,
}

#[tool_router]
impl NexusServer {
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Increment the counter by 1 and return the new value")]
    async fn increment(&self) -> Result<CallToolResult, ErrorData> {
        let mut counter = self.counter.lock().await;
        *counter += 1;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Counter incremented to: {}",
            *counter
        ))]))
    }

    #[tool(description = "Get the current counter value")]
    async fn get_counter(&self) -> Result<CallToolResult, ErrorData> {
        let counter = self.counter.lock().await;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Current counter value: {}",
            *counter
        ))]))
    }

    #[tool(description = "Reset the counter to zero")]
    async fn reset_counter(&self) -> Result<CallToolResult, ErrorData> {
        let mut counter = self.counter.lock().await;
        *counter = 0;

        Ok(CallToolResult::success(vec![Content::text(
            "Counter reset to 0".to_string(),
        )]))
    }

    #[tool(
        description = "Get company info. Finds companies with the given substring in one of its names."
    )]
    async fn get_company_info(
        &self,
        params: Parameters<CompanyInfoRequest>,
    ) -> Result<CallToolResult, ErrorData> {
        let s = params.0.search_substring;
        // Implement the logic to retrieve company information
        let data = tools::sec::get_company_substring_search(&s).unwrap();
        let result = Content::json(data);
        Ok(CallToolResult::structured(json!(result)))
    }
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CompanyInfoRequest {
    pub search_substring: String,
}

#[tool_handler]
impl ServerHandler for NexusServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation::from_build_env(),
            instructions: Some(
                "Nexus MCP Server - Advanced tool routing and state management. \
                 Available tools: increment, get_counter, reset_counter. \
                 This server demonstrates stateful operations and async tool handling."
                    .to_string(),
            ),
        }
    }
}
