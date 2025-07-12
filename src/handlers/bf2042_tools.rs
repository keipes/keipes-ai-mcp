use super::tool_handler::ToolTrait;
use bf2042_stats::StatsClient;
use futures::TryStreamExt;
use rmcp::model::*;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct WeaponsByCategoryTool {
    stats_client: Arc<StatsClient>,
}

impl WeaponsByCategoryTool {
    pub async fn new() -> Result<Self, ErrorData> {
        let stats_client = StatsClient::new().await.map_err(|e| ErrorData {
            code: ErrorCode(-32603),
            message: format!("Failed to initialize BF2042 stats client: {}", e).into(),
            data: None,
        })?;

        Ok(Self {
            stats_client: Arc::new(stats_client),
        })
    }
}

impl ToolTrait for WeaponsByCategoryTool {
    fn name(&self) -> &str {
        "bf2042_weapons_by_category"
    }

    fn description(&self) -> &str {
        "Get all weapons in a specific category (assault_rifle, sniper_rifle, etc.)"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "category": {
                    "type": "string",
                    "description": "Weapon category (e.g., assault_rifle, sniper_rifle, lmg, smg, shotgun, marksman_rifle)"
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

        Box::pin(async move {
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

            let weapons: Vec<_> = stats_client
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
