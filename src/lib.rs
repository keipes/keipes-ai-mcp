pub mod client;
pub mod logs;
pub mod server;
use crate::server::mcp;
use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use std::sync::atomic::Ordering;
use std::sync::LazyLock;
use std::sync::{atomic::AtomicUsize, Arc};
use tokio::sync::RwLock;
use tracing::info;

pub async fn run_server() {
    let nexus_service = mcp::create_nexus_service();
    let router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .fallback_service(nexus_service)
        .layer(middleware::from_fn(log_requests));
    let _ = METRICS.clone();
    server::serve(router).await;
}

struct Metrics {
    request_counter: AtomicUsize,
    sum_request_time_ms: AtomicUsize,
    min_request_time_ms: AtomicUsize,
    max_request_time_ms: AtomicUsize,
}

impl Metrics {
    fn new() -> Self {
        Metrics {
            request_counter: AtomicUsize::new(0),
            sum_request_time_ms: AtomicUsize::new(0),
            min_request_time_ms: AtomicUsize::new(usize::MAX),
            max_request_time_ms: AtomicUsize::new(0),
        }
    }

    fn log_request(&self) {
        self.request_counter.fetch_add(1, Ordering::Relaxed);
    }

    fn push_timing(&self, duration: usize) {
        self.sum_request_time_ms
            .fetch_add(duration, Ordering::Relaxed);
        self.min_request_time_ms
            .fetch_min(duration, Ordering::Relaxed);
        self.max_request_time_ms
            .fetch_max(duration, Ordering::Relaxed);
    }
}

fn to_millis(nanos: usize) -> String {
    let millis = nanos as f64 / 1_000_000.0;
    format!("{:.4}", millis)
}

static METRICS: LazyLock<Arc<Metrics>> = LazyLock::new(|| {
    let metrics = Arc::new(Metrics::new());
    info!("Metrics agent started");
    tokio::spawn({
        let metrics = metrics.clone();
        async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                let sum = metrics.sum_request_time_ms.swap(0, Ordering::Relaxed);
                let min = metrics
                    .min_request_time_ms
                    .swap(usize::MAX, Ordering::Relaxed);
                let max = metrics.max_request_time_ms.swap(0, Ordering::Relaxed);
                let processed = metrics.request_counter.swap(0, Ordering::Relaxed);
                let avg = if processed > 0 { sum / processed } else { 0 };
                if processed > 0 {
                    info!(
                        "Processed {} requests, timings (ms) total: {}, avg: {}, min: {}, max: {}",
                        processed,
                        to_millis(sum),
                        to_millis(avg),
                        to_millis(if min == usize::MAX { 0 } else { min }),
                        to_millis(max)
                    );
                }
            }
        }
    });
    metrics
});

async fn log_requests(req: Request, next: Next) -> Response {
    info!("{} Request received: {}", req.method(), req.uri());
    METRICS.log_request();
    let start = std::time::Instant::now();
    let response = next.run(req).await;
    let duration = start.elapsed();
    METRICS.push_timing(duration.as_nanos() as usize);
    info!("{} Response sent", response.status());
    response
}
