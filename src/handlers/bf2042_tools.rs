use super::tool_handler::ToolTrait;
use bf2042_stats::{DatabaseConfig, StatsClient};
use futures::TryStreamExt;
use rmcp::model::*;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct WeaponsByCategoryTool {
    stats_client: Arc<std::sync::Mutex<Option<Arc<StatsClient>>>>,
    ready: Arc<std::sync::atomic::AtomicBool>,
}

impl WeaponsByCategoryTool {
    pub fn new() -> Self {
        eprintln!("Creating new WeaponsByCategoryTool instance");
        let stats_client = Arc::new(std::sync::Mutex::new(None));
        let ready = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stats_client_clone = Arc::clone(&stats_client);
        let ready_clone = Arc::clone(&ready);
        // Spawn background task to initialize the client
        tokio::spawn(async move {
            eprintln!("Starting BF2042 stats client initialization...");
            let database_url = std::env::var("DATABASE_URL")
                .unwrap();
            eprintln!("Using database URL: {}", database_url);
            let config = DatabaseConfig::new(database_url)
                .with_max_connections(10);
            eprintln!("Created database config, attempting connection...");
            match StatsClient::new(&config).await {
                Ok(client) => {
                    eprintln!("BF2042 stats client initialized successfully");
                    *stats_client_clone.lock().unwrap() = Some(Arc::new(client));
                    ready_clone.store(true, std::sync::atomic::Ordering::SeqCst);
                    eprintln!("BF2042 stats client marked as ready");
                }
                Err(e) => {
                    eprintln!("Failed to initialize BF2042 stats client: {}", e);
                    eprintln!("Error details: {:?}", e);
                    // Remain not ready
                }
            }
        });
        eprintln!("WeaponsByCategoryTool created, background initialization started");
        Self { stats_client, ready }
    }
}

impl ToolTrait for WeaponsByCategoryTool {
    fn name(&self) -> &str {
        "bf2042_weapons_by_category"
    }

    fn description(&self) -> &str {
        "Get all weapons in a specific category [Sidearms|SMG|Assault Rifles|LMG|DMR|Bolt Action|Shotgun/Utility]"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "category": {
                    "type": "string",
                    "description": "Weapon category [Sidearms|SMG|Assault Rifles|LMG|DMR|Bolt Action|Shotgun/Utility]",
                }
            },
            "required": ["category"]
        })
    }

    fn execute(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Pin<Box<dyn Future<Output = Result<CallToolResult, ErrorData>> + Send + '_>> {
        let stats_client = Arc::clone(&self.stats_client);
        let ready = Arc::clone(&self.ready);
        Box::pin(async move {
            if !ready.load(std::sync::atomic::Ordering::SeqCst) {
                return Err(ErrorData {
                    code: ErrorCode(-32603),
                    message: "BF2042 stats client is still initializing. Try again later.".into(),
                    data: None,
                });
            }
            let args = arguments.ok_or_else(|| ErrorData {
                code: ErrorCode(-32602),
                message: "Arguments required".into(),
                data: None,
            })?;
            let category = args
                .get("category")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ErrorData {
                    code: ErrorCode(-32602),
                    message: "Category parameter required".into(),
                    data: None,
                })?;
            let client = {
                let client_guard = stats_client.lock().unwrap();
                client_guard.as_ref().ok_or_else(|| ErrorData {
                    code: ErrorCode(-32603),
                    message: "BF2042 stats client is not available.".into(),
                    data: None,
                })?.clone()
            };
            let weapons: Vec<_> = client
                .weapons_by_category(category)
                .try_collect()
                .await
                .map_err(|e| ErrorData {
                    code: ErrorCode(-32603),
                    message: format!("Failed to get weapons: {}", e).into(),
                    data: None,
                })?;
            let result_text = serde_json::to_string_pretty(&weapons)
                .unwrap_or_else(|_| "Failed to serialize weapons".to_string());
            Ok(CallToolResult {
                content: vec![Content::text(&result_text)],
                is_error: Some(false),
            })
        })
    }
}
