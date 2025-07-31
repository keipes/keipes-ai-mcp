use std::env;

use anyhow::Result;
use dotenvy::dotenv;
use keipes_ai_mcp::{client::stress, logs};

#[tokio::main]
async fn main() -> Result<()> {
    // let last = Arc::new(AtomicU64::new(0));
    // let delta_layer = fmt::layer().event_format(DeltaTimeFormatter { last: last.clone() });
    // tracing_subscriber::fmt()
    //     .with_span_events(FmtSpan::CLOSE)
    //     .with_env_filter(
    //         tracing_subscriber::EnvFilter::from_default_env()
    //             .add_directive("info".parse()?)
    //             .add_directive("rmcp=trace".parse()?)
    //             .add_directive("main=debug".parse()?),
    //     )
    //     .with_timer(tracing_subscriber::fmt::time::uptime()) // Shows time since program start
    //     .with(delta_layer)
    //     // .json()
    //     .init();

    // let last = Arc::new(AtomicU64::new(0));
    // let delta_layer = fmt::layer().event_format(DeltaTimeFormatter { last: last.clone() });
    // let env_filter = EnvFilter::from_default_env()
    //     .add_directive("info".parse()?)
    //     .add_directive("rmcp=trace".parse()?)
    //     .add_directive("main=debug".parse()?);

    // Registry::default()
    //     .with(env_filter)
    //     // .with(fmt::layer().with_timer(tracing_subscriber::fmt::time::uptime()))
    //     .with(delta_layer)
    //     .init();

    // logs::init_logging()?;
    logs::init_logging().map_err(|e| anyhow::Error::msg(e.to_string()))?;
    dotenv()?;
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
    Ok(())
}
