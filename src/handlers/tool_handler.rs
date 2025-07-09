use rmcp::model::*;
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoRequest {
    pub text: String,
}

pub struct ToolHandler;

impl ToolHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn list_tools(&self, _request: Option<PaginatedRequestParam>) -> ListToolsResult {
        let tools = vec![
            Tool {
                name: "echo".into(),
                description: Some("Echo the input text back".into()),
                input_schema: Arc::new({
                    let mut schema = serde_json::Map::new();
                    schema.insert("type".to_string(), serde_json::Value::String("object".to_string()));
                    
                    let mut properties = serde_json::Map::new();
                    let mut text_prop = serde_json::Map::new();
                    text_prop.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                    text_prop.insert("description".to_string(), serde_json::Value::String("The text to echo back".to_string()));
                    properties.insert("text".to_string(), serde_json::Value::Object(text_prop));
                    
                    schema.insert("properties".to_string(), serde_json::Value::Object(properties));
                    schema.insert("required".to_string(), serde_json::Value::Array(vec![serde_json::Value::String("text".to_string())]));
                    
                    schema
                }),
                annotations: None,
            }
        ];
        
        ListToolsResult { 
            tools,
            next_cursor: None,
        }
    }

    pub async fn call_tool(&self, request: CallToolRequestParam) -> Result<CallToolResult, ErrorData> {
        match request.name.as_ref() {
            "echo" => {
                let text = request.arguments
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
            }
            _ => Err(ErrorData {
                code: ErrorCode(-32601),
                message: format!("Tool '{}' not found", request.name).into(),
                data: None,
            }),
        }
    }

    pub async fn echo_tool(&self, text: String) -> String {
        text
    }
}
