use axum::Router;
use axum::ServiceExt;
use axum_server::tls_rustls::RustlsConfig;
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use std::{env, net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;
use tracing::{error, info};

use crate::mcp;

// Start the server by binding to a socket and serving the router
async fn serve_https(
    listener: TcpListener,
    config: RustlsConfig,
    router: Router,
) -> Result<(), Box<dyn std::error::Error>> {
    let std_listener = listener.into_std()?;
    info!("Binding to address: {}", std_listener.local_addr()?);
    let service = router.into_make_service();
    axum_server::from_tcp_rustls(std_listener, config)
        .serve(service)
        .await
        .map_err(|e| e.into())
}

// async fn serve_http(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
//     let addr = SocketAddr::from_str(addr)?;
//     info!("Binding to address: {}", addr);
//     let nexus_service = mcp::create_nexus_service();
//     axum_server::bind(addr)
//         .serve(nexus_service.into_make_service())
//         .await
//         .map_err(|e| e.into())
// }

async fn serve_http(addr: &str, router: Router) -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from_str(addr)?;
    info!("Binding to address: {}", addr);
    // let nexus_service = mcp::create_nexus_service();
    // let nexus_make_service = nexus_service.into_make_service();
    axum_server::bind(addr)
        // .serve(nexus_service.into_make_service())
        // .serve(nexus_make_service)
        .serve(router.into_make_service())
        .await
        .map_err(|e| e.into())
}

async fn get_tls_config() -> Result<RustlsConfig, String> {
    let cert_path = env::var("TLS_CERT").map_err(|_| "TLS_CERT missing")?;
    let key_path = env::var("TLS_KEY").map_err(|_| "TLS_KEY missing")?;
    check_file(&cert_path);
    check_file(&key_path);
    let config = RustlsConfig::from_pem_file(cert_path, key_path)
        .await
        .map_err(|e| format!("Failed to load cert/key: {}", e));
    config
}

fn check_file(path: &str) {
    if !std::path::Path::new(path).exists() {
        error!("File not found: {}", path);
    }
    // check can access the file
    if let Err(e) = std::fs::metadata(path) {
        error!("Cannot access file {}: {}", path, e);
    }
}

async fn get_listener(addr: &str) -> Result<TcpListener, String> {
    let addr = addr
        .parse::<SocketAddr>()
        .map_err(|e| format!("Invalid address: {}", e))?;
    TcpListener::bind(addr)
        .await
        .map_err(|e| format!("Failed to bind to address {}: {}", addr, e))
}

pub async fn serve(router: Router) {
    let http_server = tokio::spawn({
        let router = router.clone();
        async move {
            let port = env::var("HTTP_PORT").unwrap_or_else(|_| "9080".to_string());
            let addr = format!("0.0.0.0:{}", port);
            if let Err(e) = serve_http(&addr, router).await {
                error!("HTTP server failed: {}", e);
            }
        }
    });
    let https_server = tokio::spawn({
        let router = router.clone();
        async move {
            let port = env::var("HTTPS_PORT").unwrap_or_else(|_| "9443".to_string());
            let addr = format!("0.0.0.0:{}", port);
            let (listener, config) = tokio::join!(get_listener(&addr), get_tls_config());
            match (listener, config) {
                (Ok(listener), Ok(config)) => {
                    if let Err(e) = serve_https(listener, config, router).await {
                        error!("HTTPS server failed: {}", e);
                    }
                }
                (Err(e), _) => error!("Failed to create HTTPS listener: {}", e),
                (_, Err(e)) => error!("Failed to load TLS config: {}", e),
            }
        }
    });
    let mut handles = FuturesUnordered::from_iter(vec![http_server, https_server]);
    while let Some(result) = handles.next().await {
        if let Err(e) = result {
            error!("Server task failed: {}", e);
        }
        info!("Server task exited");
    }
}
