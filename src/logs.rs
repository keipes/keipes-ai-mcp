use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use tracing::Event;
use tracing::Subscriber;
use tracing_subscriber::fmt;
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::Registry;
use tracing_subscriber::EnvFilter;

pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    let last = Arc::new(AtomicU64::new(0));
    let delta_layer = fmt::layer().event_format(DeltaTimeFormatter { last: last.clone() });
    let env_filter = EnvFilter::from_default_env()
        .add_directive("trace".parse()?)
        .add_directive("h2::proto::streams=debug".parse()?)
        .add_directive("h2::proto::connection=trace".parse()?)
        .add_directive("h2::hpack=debug".parse()?)
        // .add_directive("h2::codec=info".parse()?)
        // .add_directive("h2::frame=debug".parse()?)
        .add_directive("rustls::client::hs=debug".parse()?)
        .add_directive("rmcp=trace".parse()?)
        .add_directive("main=debug".parse()?);

    Registry::default()
        .with(env_filter)
        .with(delta_layer)
        .with(fmt::layer())
        // .with(fmt::layer().with_timer(tracing_subscriber::fmt::time::uptime()))
        .init();
    Ok(())
}

pub struct DeltaTimeFormatter {
    last: Arc<AtomicU64>,
}

impl<S, N> FormatEvent<S, N> for DeltaTimeFormatter
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _: &FmtContext<'_, S, N>,
        mut writer: fmt::format::Writer<'_>,
        _: &Event<'_>,
    ) -> std::fmt::Result {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let last = self.last.swap(now, Ordering::Relaxed);
        let delta = if last == 0 { 0 } else { now - last };
        write!(writer, "[{}] ", delta)?;
        // fmt::format::DefaultFields::new().format_fields(writer.by_ref(), event)?;
        // writeln!(writer)?;
        Ok(())
    }
}
