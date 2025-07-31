//! Nexus MCP Server
//!
//! A high-performance Model Context Protocol (MCP) server implementation
//! providing advanced tool routing and capability management.

use std::{future::Future, sync::Arc, time::Duration};
use tokio::sync::Mutex;

use rmcp::{
    handler::server::tool::ToolRouter,
    model::*,
    tool, tool_handler, tool_router,
    transport::{streamable_http_server::StreamableHttpService, StreamableHttpServerConfig},
    ServerHandler,
};

/// Creates a new Nexus MCP service instance configured for HTTP transport.
///
/// # Returns
/// A configured `StreamableHttpService` ready to be embedded in a web server.
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

/// Nexus MCP Server - A sophisticated tool-enabled MCP server implementation.
///
/// Provides intelligent tool routing, state management, and extensible capabilities
/// for Model Context Protocol interactions.
#[derive(Clone)]
pub struct NexusServer {
    /// Internal counter state for demonstration tools
    counter: Arc<Mutex<i64>>,
    /// Tool router for handling and dispatching tool calls
    tool_router: ToolRouter<NexusServer>,
}

#[tool_router]
impl NexusServer {
    /// Creates a new Nexus server instance with initialized state.
    pub fn new() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
            tool_router: Self::tool_router(),
        }
    }

    /// Increments the internal counter and returns the new value.
    ///
    /// This demonstrates stateful tool operations and async tool handling.
    #[tool(description = "Increment the counter by 1 and return the new value")]
    async fn increment(&self) -> Result<CallToolResult, ErrorData> {
        let mut counter = self.counter.lock().await;
        *counter += 1;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Counter incremented to: {}",
            *counter
        ))]))
    }

    /// Retrieves the current counter value without modification.
    #[tool(description = "Get the current counter value")]
    async fn get_counter(&self) -> Result<CallToolResult, ErrorData> {
        let counter = self.counter.lock().await;

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Current counter value: {}",
            *counter
        ))]))
    }

    /// Resets the counter to zero.
    #[tool(description = "Reset the counter to zero")]
    async fn reset_counter(&self) -> Result<CallToolResult, ErrorData> {
        let mut counter = self.counter.lock().await;
        *counter = 0;

        Ok(CallToolResult::success(vec![Content::text(
            "Counter reset to 0".to_string(),
        )]))
    }
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
