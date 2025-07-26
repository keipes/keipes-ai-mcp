use anyhow::Result;
use rmcp::{
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    transport::StreamableHttpClientTransport,
    ServiceExt,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // let client = NexusClient::new().await?;
    // client.demo().await?;
    Ok(())
}
