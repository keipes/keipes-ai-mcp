use console_subscriber::init as tokio_console_init;
use dotenvy::dotenv;
use keipes_ai_mcp::{logs, run_server};
// use keipes_ai_mcp::McpServer;
use std::env;
use tracing::{info, warn};

static TOKIO_CONSOLE: bool = false;

// #[tokio::main(flavor = "current_thread")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    if TOKIO_CONSOLE {
        tokio_console_init();
    } else {
        // tracing_subscriber::fmt()
        //     .with_env_filter(
        //         tracing_subscriber::EnvFilter::from_default_env()
        //             .add_directive("debug".parse()?)
        //             .add_directive("rmcp=debug".parse()?)
        //             .add_directive("main=debug".parse()?)
        //             .add_directive("bf2042_stats=debug".parse()?),
        //     )
        //     // .json()
        //     .init();
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
