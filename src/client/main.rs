use std::env;

use anyhow::Result;
use keipes_ai_mcp::{client::stress, logs};

#[cfg(feature = "dhat-heap")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() -> Result<()> {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    logs::init_logging().map_err(|e| anyhow::Error::msg(e.to_string()))?;
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 4 {
        eprintln!("Usage: {} <server_uri> <workers> <number>", args[0]);
        return Ok(());
    }
    let server_uri = &args[1];
    let workers = &args[2];
    let number = &args[3];
    tracing::info!("Starting stress test on server: {}", server_uri);
    stress(server_uri, workers.parse()?, number.parse()?).await?;
    Ok(())
}
