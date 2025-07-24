pub mod handlers;
pub mod mcp;
pub mod tools;
pub mod types;

use crate::handlers::server;
use axum::{
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use tracing::info;
use types::ServerConfig;

#[derive(Clone)]
pub struct McpServer {
    config: ServerConfig,
}

impl McpServer {
    pub async fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    pub async fn start(&self) -> Result<(), String> {
        info!(
            "Starting MCP Server on {}:{}",
            self.config.bind_address, self.config.port
        );

        Ok(())
    }

    pub async fn serve(&self) {
        let server = self.clone();
        let nexus_service = mcp::create_nexus_service();
        let app = Router::new()
            // .route("/mcp", post(Self::handle_post).get(Self::handle_get))
            .nest_service("/mcp", nexus_service)
            .route("/health", get(|| async { "OK" }))
            // default 404 status for unmatched routes or methods
            .fallback(get(|| async { StatusCode::NOT_FOUND }))
            // fallback for post
            .fallback(post(|| async { StatusCode::NOT_FOUND }))
            .layer(middleware::from_fn(Self::log_requests))
            .with_state(server);
        let addr = format!("{}:{}", self.config.bind_address, self.config.port);
        info!("MCP Server listening on {}", addr);
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
