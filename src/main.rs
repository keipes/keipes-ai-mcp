use keipes_ai_mcp::{McpServer, types::ServerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ServerConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 8000,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
    };

    let server = McpServer::new(config);
    server.start().await?;

    Ok(())
}
