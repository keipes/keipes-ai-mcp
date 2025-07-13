use keipes_ai_mcp::{types::ServerConfig, McpServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig {
        bind_address: "0.0.0.0".to_string(),
        port: 80,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
    };

    let server = McpServer::new(config).await;
    server.start().await?;

    Ok(())
}
