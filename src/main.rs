// use console_subscriber::init as tokio_console_init;
use dotenvy::dotenv;
use keipes_ai_mcp::{types::ServerConfig, McpServer};
use std::env;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // tokio_console_init();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse()?)
                .add_directive("main=debug".parse()?)
                .add_directive("bf2042_stats=debug".parse()?),
        )
        .json()
        .init();
    let _ = rustls::crypto::ring::default_provider().install_default();
    // handle dotenv errors gracefully
    let got_env = dotenv().ok();
    if let Some(env_file) = got_env {
        info!(
            "Loaded environment variables from: {}",
            env_file.as_path().display()
        );
    } else {
        warn!(
            "No .env file found at {}, using default environment variables",
            env::current_dir().unwrap().display()
        );
    }
    let build_time = env::var("BUILD_TIME").unwrap_or_else(|_| "unknown".to_string());
    info!(
        "Starting the MCP server with tracing enabled, build time: {}",
        build_time
    );
    let _ = axum_server().await.serve().await;

    Ok(())
}

async fn axum_server() -> McpServer {
    McpServer::new().await
}
