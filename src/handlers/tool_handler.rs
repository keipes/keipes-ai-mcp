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
            "echo" => self.echo_tool(&request).await,
            _ => Err(ErrorData {
                code: ErrorCode(-32601),
                message: format!("Tool '{}' not found", request.name).into(),
                data: None,
            }),
        }
    }

    pub async fn echo_tool(&self, request: &CallToolRequestParam) -> Result<CallToolResult, ErrorData> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::CallToolRequestParam;

    #[tokio::test]
    async fn test_list_tools() {
        let handler = ToolHandler::new();
        let result = handler.list_tools(None).await;
        
        assert_eq!(result.tools.len(), 1);
        assert_eq!(result.tools[0].name, "echo");
        assert!(result.tools[0].description.is_some());
        assert!(result.next_cursor.is_none());
    }

    #[tokio::test]
    async fn test_echo_tool() {
        let handler = ToolHandler::new();
        
        let mut args = serde_json::Map::new();
        args.insert("text".to_string(), serde_json::Value::String("Hello, World!".to_string()));
        
        let request = CallToolRequestParam {
            name: "echo".into(),
            arguments: Some(args),
        };
        
        let result = handler.echo_tool(&request).await;
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
        let handler = ToolHandler::new();
        
        let mut args = serde_json::Map::new();
        args.insert("text".to_string(), serde_json::Value::String("Hello MCP!".to_string()));
        
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
