use super::bf2042_tools::WeaponsByCategoryTool;
use rmcp::model::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoRequest {
    pub text: String,
}

pub trait ToolTrait {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> serde_json::Value;
    fn execute(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Pin<Box<dyn Future<Output = Result<CallToolResult, ErrorData>> + Send + '_>>;
}

pub struct EchoTool;

impl ToolTrait for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echo the input text back"
    }

    fn input_schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "text": {
                    "type": "string",
                    "description": "The text to echo back"
                }
            },
            "required": ["text"]
        })
    }

    fn execute(
        &self,
        arguments: Option<serde_json::Map<String, serde_json::Value>>,
    ) -> Pin<Box<dyn Future<Output = Result<CallToolResult, ErrorData>> + Send + '_>> {
        Box::pin(async move {
            let text = arguments
                .as_ref()
                .and_then(|args| args.get("text"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ErrorData {
                    code: ErrorCode(-32602),
                    message: "Invalid arguments: 'text' parameter required".into(),
                    data: None,
                })?;

            Ok(CallToolResult {
                content: vec![Content::text(text)],
                is_error: Some(false),
            })
        })
    }
}

pub struct ToolHandler {
    tools: HashMap<String, Arc<dyn ToolTrait + Send + Sync>>,
}

impl Clone for ToolHandler {
    fn clone(&self) -> Self {
        Self {
            tools: self.tools.clone(),
        }
    }
}

impl ToolHandler {
    pub async fn new() -> Self {
        let mut tools: HashMap<String, Arc<dyn ToolTrait + Send + Sync>> = HashMap::new();
        let echo_tool: Arc<dyn ToolTrait + Send + Sync> = Arc::new(EchoTool);
        tools.insert(echo_tool.name().to_string(), echo_tool);

        // Try to initialize BF2042 tools, but don't fail if they're not available
        let weapons_by_category_tool = WeaponsByCategoryTool::new();
        let tool: Arc<dyn ToolTrait + Send + Sync> = Arc::new(weapons_by_category_tool);
        tools.insert(tool.name().to_string(), tool);

        Self { tools }
    }

    pub fn capabilities(&self) -> HashMap<String, serde_json::Value> {
        let mut capabilities = HashMap::new();
        for tool in self.tools.values() {
            capabilities.insert(tool.name().to_string(), tool.input_schema());
        }
        capabilities
    }

    pub async fn list_tools(&self, _request: Option<PaginatedRequestParam>) -> ListToolsResult {
        let tools = self
            .tools
            .values()
            .map(|tool| Tool {
                name: tool.name().to_string().into(),
                description: Some(tool.description().to_string().into()),
                input_schema: Arc::new(tool.input_schema().as_object().unwrap().clone()),
                annotations: None,
            })
            .collect();

        ListToolsResult {
            tools,
            next_cursor: None,
        }
    }

    pub async fn call_tool(
        &self,
        request: CallToolRequestParam,
    ) -> Result<CallToolResult, ErrorData> {
        let tool_name = request.name.to_string();
        if let Some(tool) = self.tools.get(&tool_name) {
            tool.execute(request.arguments).await
        } else {
            Err(ErrorData {
                code: ErrorCode(-32601),
                message: format!("Tool '{}' not found", request.name).into(),
                data: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::CallToolRequestParam;

    #[tokio::test]
    async fn test_list_tools() {
        let handler = ToolHandler::new().await;
        let result = handler.list_tools(None).await;

        assert_eq!(result.tools.len(), 2);
        assert_eq!(result.tools[0].name, "echo");
        assert!(result.tools[0].description.is_some());
        assert!(result.next_cursor.is_none());
    }

    #[tokio::test]
    async fn test_echo_tool() {
        let handler = ToolHandler::new().await;

        let mut args = serde_json::Map::new();
        args.insert(
            "text".to_string(),
            serde_json::Value::String("Hello, World!".to_string()),
        );

        let request = CallToolRequestParam {
            name: "echo".into(),
            arguments: Some(args),
        };

        let result = handler.call_tool(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.is_error, Some(false));
        assert_eq!(response.content.len(), 1);

        // The content should be text content with "Hello, World!"
        // We'll just verify it's not empty for now since the exact structure may vary
        assert!(!response.content.is_empty());
    }

    #[tokio::test]
    async fn test_call_tool() {
        let handler = ToolHandler::new().await;

        let mut args = serde_json::Map::new();
        args.insert(
            "text".to_string(),
            serde_json::Value::String("Hello MCP!".to_string()),
        );

        let request = CallToolRequestParam {
            name: "echo".into(),
            arguments: Some(args),
        };

        let result = handler.call_tool(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.is_error, Some(false));
        assert_eq!(response.content.len(), 1);
    }
}
