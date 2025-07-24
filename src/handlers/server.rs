use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;
use std::{env, net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;
use tracing::{error, info};

// Start the server by binding to a socket and serving the router
async fn serve_https(listener: TcpListener, config: RustlsConfig, router: Router) {
    // let addr = SocketAddr::from_str(addr).unwrap();
    let std_listener = listener.into_std().unwrap();
    info!("Binding to address: {}", std_listener.local_addr().unwrap());
    let service = router.into_make_service();
    axum_server::from_tcp_rustls(std_listener, config)
        .serve(service)
        .await
        .unwrap();
}

async fn serve_http(addr: &str, router: Router) {
    let addr = SocketAddr::from_str(addr).unwrap();
    info!("Binding to address: {}", addr);
    axum_server::bind(addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
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
            serve_http("0.0.0.0:80", router).await;
        }
    });
    let https_server = tokio::spawn({
        let router = router.clone();
        async move {
            let addr = "0.0.0.0:443";
            let (listener, config) = tokio::join!(get_listener(addr), get_tls_config());
            if let (Ok(listener), Ok(config)) = (listener, config) {
                serve_https(listener, config, router).await;
            }
        }
    });
    let mut handles = FuturesUnordered::from_iter(vec![http_server, https_server]);
    while let Some(result) = handles.next().await {
        result.unwrap();
        info!("Server task exited");
    }
}
