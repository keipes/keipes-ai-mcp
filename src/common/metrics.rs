use std::sync::atomic::Ordering;
use std::sync::LazyLock;
use std::sync::{atomic::AtomicUsize, Arc};
use tracing::info;

pub struct Metrics {
    request_counter: AtomicUsize,
    sum_request_time_ms: AtomicUsize,
    min_request_time_ms: AtomicUsize,
    max_request_time_ms: AtomicUsize,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            request_counter: AtomicUsize::new(0),
            sum_request_time_ms: AtomicUsize::new(0),
            min_request_time_ms: AtomicUsize::new(usize::MAX),
            max_request_time_ms: AtomicUsize::new(0),
        }
    }

    pub fn log_request(&self) {
        self.request_counter.fetch_add(1, Ordering::Relaxed);
    }

    pub fn push_timing(&self, duration: usize) {
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

pub static METRICS: LazyLock<Arc<Metrics>> = LazyLock::new(|| {
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
