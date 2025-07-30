use std::env;

use anyhow::Result;
use dotenvy::dotenv;
use keipes_ai_mcp::client::stress;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse()?)
                .add_directive("main=debug".parse()?),
        )
        // .json()
        .init();
    dotenv()?;
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("Usage: {} <server_uri>", args[0]);
        return Ok(());
    }
    let server_uri = &args[1];
    // let client = NexusClient::new(server_uri).await?;
    // client.demo().await?;
    tracing::info!("Starting stress test on server: {}", server_uri);
    stress(server_uri, 10, 1000).await?;
    Ok(())
}
