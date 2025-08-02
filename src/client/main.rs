use console_subscriber::init as tokio_console_init;
use std::env;

use anyhow::Result;
use keipes_ai_mcp::{client::stress, common::app_env, logs};

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<()> {
    if app_env::use_tokio_console() {
        tokio_console_init();
    } else {
        logs::init_logging().map_err(|e| anyhow::Error::msg(e.to_string()))?;
    }
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 4 {
        eprintln!("Usage: {} <server_uri> <workers> <number>", args[0]);
        return Ok(());
    }

    let server_uri = &args[1];
    let workers = &args[2];
    let number = &args[3];
    // let client = NexusClient::new(server_uri).await?;
    // client.demo().await?;
    tracing::info!("Starting stress test on server: {}", server_uri);
    stress(server_uri, workers.parse()?, number.parse()?).await?;

    // loop forever, sleep 10ms
    // loop {
    //     tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    // }
    Ok(())
}
