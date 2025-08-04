use dotenvy::dotenv;
use keipes_ai_mcp::{logs, server::run_server};
// use keipes_ai_mcp::McpServer;
use std::{env, time::Duration};
use tracing::{info, warn};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

// #[tokio::main(flavor = "current_thread")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "dhat-heap")]
    let profiler = dhat::Profiler::new_heap();

    logs::init_logging()?;
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
            "No .env file found at {}, using default environment",
            env::current_dir().unwrap().display()
        );
    }
    let build_time = env::var("BUILD_TIME").unwrap_or_else(|_| "unknown".to_string());
    info!(
        "Starting the MCP server with tracing enabled, build time: {}",
        build_time
    );

    #[cfg(feature = "dhat-heap")]
    tokio::spawn(async {
        // tokio::time::sleep(Duration::from_secs(30)).await;
        // drop(profiler);
        if let Err(e) = tokio::signal::ctrl_c().await {
            warn!("Failed to listen for Ctrl+C: {}", e);
        }
        info!("Ctrl+C received, writing dhat profile...");
        drop(profiler);
    });

    run_server().await;
    Ok(())
}
