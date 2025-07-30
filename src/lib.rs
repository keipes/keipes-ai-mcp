pub mod client;
pub mod handlers;
pub mod mcp;

use crate::handlers::server;
use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use tracing::info;

#[derive(Clone)]
pub struct McpServer {}

impl McpServer {
    pub async fn new() -> Self {
        Self {}
    }

    pub async fn serve(&self) {
        let server = self.clone();

        let nexus_service = mcp::create_nexus_service();
        let app = Router::new()
            .route("/health", get(|| async { "OK" }))
            .fallback_service(nexus_service)
            .layer(middleware::from_fn(Self::log_requests))
            .with_state(server);
        server::serve(app).await;
    }

    pub fn shutdown(&self) {
        // Graceful shutdown will be handled by dropping the server
    }

    // Request logging middleware
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
}
