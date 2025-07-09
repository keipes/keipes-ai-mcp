use rmcp::model::*;

pub struct PromptHandler;

impl PromptHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn list_prompts(&self, _request: Option<PaginatedRequestParam>) -> ListPromptsResult {
        let prompts = vec![
            Prompt {
                name: "greeting".into(),
                description: Some("A simple greeting prompt".into()),
                arguments: Some(vec![
                    PromptArgument {
                        name: "name".into(),
                        description: Some("The name to greet".into()),
                        required: Some(true),
                    }
                ]),
            }
        ];
        
        ListPromptsResult {
            prompts,
            next_cursor: None,
        }
    }

    pub async fn get_prompt(&self, request: GetPromptRequestParam) -> Result<GetPromptResult, ErrorData> {
        match request.name.as_str() {
            "greeting" => {
                let name = request.arguments
                    .as_ref()
                    .and_then(|args| args.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("World");

                Ok(GetPromptResult {
                    description: Some("A simple greeting prompt".into()),
                    messages: vec![
                        PromptMessage {
                            role: PromptMessageRole::User,
                            content: PromptMessageContent::text(&format!("Hello, {}!", name)),
                        }
                    ],
                })
            }
            _ => Err(ErrorData {
                code: ErrorCode(-32601),
                message: format!("Prompt '{}' not found", request.name).into(),
                data: None,
            }),
        }
    }
}
