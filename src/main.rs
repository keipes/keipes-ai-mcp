use keipes_ai_mcp::{types::ServerConfig, McpServer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing to capture logs from bf2042_stats and other dependencies
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env().add_directive("bf2042_stats=debug".parse()?))
        .init();

    eprintln!("Starting MCP server with tracing enabled");

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
