use rmcp::model::*;
use std::sync::Arc;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

pub trait PromptTrait {
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn arguments(&self) -> Option<Vec<PromptArgument>>;
    fn execute(&self, arguments: Option<serde_json::Map<String, serde_json::Value>>) -> Pin<Box<dyn Future<Output = Result<GetPromptResult, ErrorData>> + Send + '_>>;
}

pub struct GreetingPrompt;

impl PromptTrait for GreetingPrompt {
    fn name(&self) -> &str {
        "greeting"
    }

    fn description(&self) -> Option<&str> {
        Some("A simple greeting prompt")
    }

    fn arguments(&self) -> Option<Vec<PromptArgument>> {
        Some(vec![
            PromptArgument {
                name: "name".into(),
                description: Some("The name to greet".into()),
                required: Some(true),
            }
        ])
    }

    fn execute(&self, arguments: Option<serde_json::Map<String, serde_json::Value>>) -> Pin<Box<dyn Future<Output = Result<GetPromptResult, ErrorData>> + Send + '_>> {
        let description = self.description().map(|d| d.into());
        Box::pin(async move {
            let name = arguments
                .as_ref()
                .and_then(|args| args.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("World");

            Ok(GetPromptResult {
                description,
                messages: vec![
                    PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::text(&format!("Hello, {}!", name)),
                    }
                ],
            })
        })
    }
}

pub struct PromptHandler {
    prompts: HashMap<String, Arc<dyn PromptTrait + Send + Sync>>,
}

impl Clone for PromptHandler {
    fn clone(&self) -> Self {
        Self {
            prompts: self.prompts.clone(),
        }
    }
}

impl PromptHandler {
    pub fn new() -> Self {
        let mut prompts: HashMap<String, Arc<dyn PromptTrait + Send + Sync>> = HashMap::new();
        let greeting_prompt: Arc<dyn PromptTrait + Send + Sync> = Arc::new(GreetingPrompt);
        prompts.insert(greeting_prompt.name().to_string(), greeting_prompt);
        
        Self { prompts }
    }

    pub fn capabilities(&self) -> HashMap<String, serde_json::Value> {
        self.prompts.iter().map(|(name, prompt)| {
            (name.clone(), serde_json::json!({
                "description": prompt.description(),
                "arguments": prompt.arguments()
            }))
        }).collect()
    }

    pub async fn list_prompts(&self, _request: Option<PaginatedRequestParam>) -> ListPromptsResult {
        let prompts = self.prompts.values().map(|prompt| {
            Prompt {
                name: prompt.name().into(),
                description: prompt.description().map(|d| d.into()),
                arguments: prompt.arguments(),
            }
        }).collect();
        
        ListPromptsResult {
            prompts,
            next_cursor: None,
        }
    }

    pub async fn get_prompt(&self, request: GetPromptRequestParam) -> Result<GetPromptResult, ErrorData> {
        if let Some(prompt) = self.prompts.get(&request.name) {
            prompt.execute(request.arguments).await
        } else {
            Err(ErrorData {
                code: ErrorCode(-32601),
                message: format!("Prompt '{}' not found", request.name).into(),
                data: None,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::model::GetPromptRequestParam;

    #[tokio::test]
    async fn test_greeting_prompt_name() {
        let prompt = GreetingPrompt;
        assert_eq!(prompt.name(), "greeting");
    }

    #[tokio::test]
    async fn test_greeting_prompt_description() {
        let prompt = GreetingPrompt;
        assert_eq!(prompt.description(), Some("A simple greeting prompt"));
    }

    #[tokio::test]
    async fn test_greeting_prompt_arguments() {
        let prompt = GreetingPrompt;
        let args = prompt.arguments().unwrap();
        assert_eq!(args.len(), 1);
        assert_eq!(args[0].name, "name");
        assert_eq!(args[0].required, Some(true));
    }

    #[tokio::test]
    async fn test_greeting_prompt_execute_with_name() {
        let prompt = GreetingPrompt;
        let mut args = serde_json::Map::new();
        args.insert("name".to_string(), serde_json::Value::String("Alice".to_string()));
        
        let result = prompt.execute(Some(args)).await.unwrap();
        assert!(result.messages.len() > 0);
        assert_eq!(result.messages[0].role, PromptMessageRole::User);
    }

    #[tokio::test]
    async fn test_greeting_prompt_execute_without_name() {
        let prompt = GreetingPrompt;
        let result = prompt.execute(None).await.unwrap();
        assert!(result.messages.len() > 0);
        assert_eq!(result.messages[0].role, PromptMessageRole::User);
    }

    #[test]
    fn test_prompt_handler_new() {
        let handler = PromptHandler::new();
        assert_eq!(handler.prompts.len(), 1);
        assert!(handler.prompts.contains_key("greeting"));
    }

    #[test]
    fn test_prompt_handler_capabilities() {
        let handler = PromptHandler::new();
        let caps = handler.capabilities();
        assert_eq!(caps.len(), 1);
        assert!(caps.contains_key("greeting"));
    }

    #[tokio::test]
    async fn test_prompt_handler_list_prompts() {
        let handler = PromptHandler::new();
        let result = handler.list_prompts(None).await;
        assert_eq!(result.prompts.len(), 1);
        assert_eq!(result.prompts[0].name, "greeting");
        assert!(result.next_cursor.is_none());
    }

    #[tokio::test]
    async fn test_prompt_handler_get_prompt_success() {
        let handler = PromptHandler::new();
        let request = GetPromptRequestParam {
            name: "greeting".to_string(),
            arguments: None,
        };
        let result = handler.get_prompt(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_prompt_handler_get_prompt_not_found() {
        let handler = PromptHandler::new();
        let request = GetPromptRequestParam {
            name: "nonexistent".to_string(),
            arguments: None,
        };
        let result = handler.get_prompt(request).await;
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code.0, -32601);
    }

    #[test]
    fn test_prompt_handler_clone() {
        let handler = PromptHandler::new();
        let cloned = handler.clone();
        assert_eq!(handler.prompts.len(), cloned.prompts.len());
    }
}
