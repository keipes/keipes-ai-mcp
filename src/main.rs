use console_subscriber::init as tokio_console_init;
use dotenvy::dotenv;
use keipes_ai_mcp::{common::app_env, logs, server::run_server};
// use keipes_ai_mcp::McpServer;
use std::env;
use tracing::{info, warn};

// #[tokio::main(flavor = "current_thread")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if app_env::use_tokio_console() {
        tokio_console_init();
    } else {
        logs::init_logging()?;
    }
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
    run_server().await;
    Ok(())
}
