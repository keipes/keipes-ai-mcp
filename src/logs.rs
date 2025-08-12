use tracing_subscriber::fmt::{self};
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::Registry;
use tracing_subscriber::EnvFilter;

use crate::common::app_env;

pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    // let sub_builder = fmt::Subscriber::builder()
    //     .with_env_filter(env_filter)
    //     .with_thread_ids(true)
    //     .with_thread_names(true)
    //     .with_file(true)
    //     .with_line_number(true)
    //     .with_target(false) // Disable target in logs
    //     .without_time() // Disable timestamp in logs
    //     .finish();

    let env_filter = EnvFilter::from_default_env()
        .add_directive("warn".parse()?)
        .add_directive("keipes_ai_mcp=info".parse()?)
        .add_directive("main=debug".parse()?);
    match app_env::use_tokio_console() {
        true => {
            tracing::info!("Using tokio console for logging");
            // let env_filter = EnvFilter::from_default_env()
            //     .add_directive("warn".parse()?)
            //     .add_directive("keipes_ai_mcp=info".parse()?)
            //     .add_directive("main=debug".parse()?);
            let console_layer = console_subscriber::ConsoleLayer::builder().spawn();
            Registry::default()
                .with(console_layer)
                .with(fmt::layer().with_filter(env_filter))
                .init()
        }
        false => {
            // let env_filter = EnvFilter::from_default_env()
            //     .add_directive("warn".parse()?)
            //     .add_directive("keipes_ai_mcp=info".parse()?)
            //     .add_directive("main=debug".parse()?);
            Registry::default()
                .with(fmt::layer().with_filter(env_filter))
                .init()
        }
    };
    Ok(())
}
