use anyhow::{Error, Result};
use futures::stream::FuturesUnordered;
use futures::stream::StreamExt;

// use lib;
use rmcp::{
    model::{
        CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation,
        InitializeRequestParam,
    },
    service::RunningService,
    transport::StreamableHttpClientTransport,
    RoleClient, ServiceExt,
};
use tracing::{error, info};
type RmcpClient = RunningService<RoleClient, InitializeRequestParam>;

// #[derive(Default)]

pub struct NexusClient {
    client: RmcpClient,
}

impl NexusClient {
    pub async fn new(server_uri: &str) -> Result<Self> {
        let client = create_client(server_uri).await?;
        Ok(Self { client })
    }

    pub async fn demo(&self) -> Result<()> {
        let tool_result = self
            .client
            .call_tool(CallToolRequestParam {
                name: "increment".into(),
                arguments: serde_json::json!({}).as_object().cloned(),
            })
            .await?;
        tracing::info!("Tool result: {tool_result:#?}");
        let server_info = self.client.peer_info();
        tracing::info!("Connected to server: {server_info:#?}");
        // List tools
        let tools = self.client.list_tools(Default::default()).await?;
        tracing::info!("Available tools: {tools:#?}");
        let tool_result = self
            .client
            .call_tool(CallToolRequestParam {
                name: "increment".into(),
                arguments: serde_json::json!({}).as_object().cloned(),
            })
            .await?;
        tracing::info!("Tool result: {tool_result:#?}");
        Ok(())
    }
}
use std::sync::Arc;
async fn create_client(server_uri: &str) -> Result<RmcpClient, Error> {
    let transport = StreamableHttpClientTransport::from_uri(server_uri);
    let client_info = ClientInfo {
        protocol_version: Default::default(),
        capabilities: ClientCapabilities::default(),
        client_info: Implementation {
            name: "test sse client".to_string(),
            version: "0.0.1".to_string(),
        },
    };
    let client = client_info.serve(transport).await.inspect_err(|e| {
        tracing::error!("client error: {:?}", e);
    })?;
    Ok(client)
}

pub async fn stress(server_uri: &str, workers: usize, total_calls: usize) -> Result<()> {
    let remaining_calls = Arc::new(std::sync::atomic::AtomicUsize::new(total_calls));
    let failures = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let successes = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    // durations is a collection of all durations
    let durations = Arc::new(std::sync::Mutex::new(Vec::new()));
    tracing::info!(
        "Starting stress test with {} workers, total calls: {}",
        workers,
        total_calls
    );
    let start = std::time::Instant::now();
    let handles = (0..workers)
        .map(|_| {
            let server_uri = server_uri.to_string();
            let remaining_calls = Arc::clone(&remaining_calls);
            let failures = Arc::clone(&failures);
            let successes = Arc::clone(&successes);
            let durations = Arc::clone(&durations);
            tokio::spawn(async move {
                let client = create_client(&server_uri)
                    .await
                    .expect("Failed to create client");
                let mut count = 0;
                tracing::info!("Worker started for server: {}", server_uri);
                while remaining_calls.load(std::sync::atomic::Ordering::SeqCst) > 0 {
                    if remaining_calls.fetch_sub(1, std::sync::atomic::Ordering::SeqCst) == 0 {
                        break;
                    }
                    count += 1;
                    let start = std::time::Instant::now();
                    let response = client
                        .call_tool(CallToolRequestParam {
                            name: "increment".into(),
                            arguments: serde_json::json!({}).as_object().cloned(),
                        })
                        .await;
                    let duration = start.elapsed().as_millis() as usize;
                    durations.lock().unwrap().push(duration);
                    if let Ok(tool_result) = response {
                        // tracing::info!("Call took {} ms", duration);
                        // tracing::info!("Tool result: {tool_result:#?}");
                        successes.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    } else {
                        tracing::error!("Error calling tool: {:?}", response);
                        failures.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    }
                }
                tracing::info!("Worker finished after {} calls", count);
            })
        })
        .collect::<Vec<_>>();
    info!("Spawned {} worker tasks", handles.len());
    let mut handles = FuturesUnordered::from_iter(handles);
    while let Some(result) = handles.next().await {
        if let Err(e) = result {
            error!("Worker task failed: {}", e);
        }
        // info!("Worker task exited");
    }
    let elapsed = start.elapsed().as_millis() as usize;
    let total_duration = durations.lock().unwrap().iter().sum::<usize>();
    let total_successes = successes.load(std::sync::atomic::Ordering::SeqCst);
    let total_failures = failures.load(std::sync::atomic::Ordering::SeqCst);
    let min_duration = durations.lock().unwrap().iter().min().cloned().unwrap_or(0);
    let max_duration = durations.lock().unwrap().iter().max().cloned().unwrap_or(0);
    let rps = if elapsed > 0 {
        (total_successes as f64 / elapsed as f64) * 1000.0
    } else {
        0.0
    };
    let mean_duration = if total_successes > 0 {
        total_duration / total_successes
    } else {
        0
    };

    info!(
        "Stress test completed: total calls: {}\n\tsuccesses: {}\n\tfailures: {}\n\telapsed: {} ms\n\tmean duration: {} ms\n\tmax duration: {} ms\n\tmin duration: {} ms\n\tRPS: {:.2}",
        total_calls, total_successes, total_failures, elapsed, mean_duration, max_duration, min_duration, rps
    );
    if total_failures > 0 {
        error!("Stress test completed with failures: {}", total_failures);
    }
    Ok(())
}
