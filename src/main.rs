use console_subscriber::init as tokio_console_init;
use dotenv::dotenv;
use keipes_ai_mcp::{types::ServerConfig, McpServer};
use std::env;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tokio_console_init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    // Initialize tracing to capture logs from bf2042_stats and other dependencies
    // tracing_subscriber::fmt()
    //     .with_env_filter(
    //         tracing_subscriber::EnvFilter::from_default_env()
    //             .add_directive("info".parse()?)
    //             .add_directive("main=debug".parse()?)
    //             .add_directive("bf2042_stats=debug".parse()?),
    //     )
    //     .json()
    //     .init();
    dotenv().ok();

    let build_time = env::var("BUILD_TIME").unwrap_or_else(|_| "unknown".to_string());
    info!(
        "Starting the MCP server with tracing enabled, build time: {}",
        build_time
    );
    let s = axum_server().await.serve().await;

    Ok(())
}

async fn axum_server() -> McpServer {
    let config = ServerConfig {
        bind_address: "0.0.0.0".to_string(),
        port: 80,
    };
    McpServer::new(config).await
}
