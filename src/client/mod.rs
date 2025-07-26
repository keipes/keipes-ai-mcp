use anyhow::{Error, Result};
// use lib;
use rmcp::{
    model::{
        CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation,
        InitializeRequestParam, InitializeResult,
    },
    service::RunningService,
    transport::StreamableHttpClientTransport,
    RoleClient, ServiceExt,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

type RmcpClient = RunningService<RoleClient, InitializeRequestParam>;

// #[derive(Default)]

pub struct NexusClient {
    client: RmcpClient,
}

impl NexusClient {
    pub async fn new() -> Result<Self> {
        let client = create_client().await?;
        Ok(Self { client })
    }

    pub async fn demo(&self) -> Result<()> {
        let tool_result = self
            .client
            .call_tool(CallToolRequestParam {
                name: "increment".into(),
                arguments: serde_json::json!({}).as_object().cloned(),
            })
            .await?;
        tracing::info!("Tool result: {tool_result:#?}");
        let server_info = self.client.peer_info();
        tracing::info!("Connected to server: {server_info:#?}");
        // List tools
        let tools = self.client.list_tools(Default::default()).await?;
        tracing::info!("Available tools: {tools:#?}");
        let tool_result = self
            .client
            .call_tool(CallToolRequestParam {
                name: "increment".into(),
                arguments: serde_json::json!({}).as_object().cloned(),
            })
            .await?;
        tracing::info!("Tool result: {tool_result:#?}");
        Ok(())
    }
}

async fn create_client() -> Result<RmcpClient, Error> {
    let transport = StreamableHttpClientTransport::from_uri("http://localhost/mcp");
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "test sse client".to_string(),
            version: "0.0.1".to_string(),
        },
    };
    let client = client_info.serve(transport).await.inspect_err(|e| {
        tracing::error!("client error: {:?}", e);
    })?;
    Ok(client)
}
