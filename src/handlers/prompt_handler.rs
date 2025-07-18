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
