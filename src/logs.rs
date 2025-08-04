use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::Registry;
use tracing_subscriber::EnvFilter;

pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let env_filter = EnvFilter::from_default_env()
        .add_directive("warn".parse()?)
        .add_directive("keipes_ai_mcp=info".parse()?)
        .add_directive("main=debug".parse()?);

    Registry::default()
        .with(env_filter)
        .with(fmt::layer())
        .init();
    Ok(())
}
