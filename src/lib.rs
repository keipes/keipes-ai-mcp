pub mod client;
pub mod server;
use crate::server::mcp;
use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use tracing::info;

pub async fn run_server() {
    let nexus_service = mcp::create_nexus_service();
    let router = Router::new()
        .route("/health", get(|| async { "OK" }))
        .fallback_service(nexus_service)
        .layer(middleware::from_fn(log_requests));
    server::serve(router).await;
}

async fn log_requests(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    if uri.path() == "/health" {
        // do not log health checks
        return next.run(req).await;
    }
    let headers = req.headers().clone();
    let origin = headers
        .get("Origin")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("none");

    info!("Request - {} {} origin: {}", method, uri, origin);

    let response = next.run(req).await;

    info!(
        "Response - {} {} status: {}",
        method,
        uri,
        response.status()
    );

    response
}
