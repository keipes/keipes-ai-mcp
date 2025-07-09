use rmcp::model::*;

pub struct ResourceHandler;

impl ResourceHandler {
    pub fn new() -> Self {
        Self
    }

    pub async fn list_resources(&self, _request: Option<PaginatedRequestParam>) -> ListResourcesResult {
        use rmcp::model::RawResource;
        
        let resources = vec![
            Resource::new(
                RawResource {
                    uri: "memory://example".into(),
                    name: "Example Resource".to_string(),
                    description: Some("An example in-memory resource".to_string()),
                    mime_type: Some("text/plain".to_string()),
                    size: None,
                },
                None
            )
        ];
        
        ListResourcesResult {
            resources,
            next_cursor: None,
        }
    }

    pub async fn read_resource(&self, request: ReadResourceRequestParam) -> Result<ReadResourceResult, ErrorData> {
        match request.uri.as_str() {
            "memory://example" => {
                Ok(ReadResourceResult {
                    contents: vec![
                        ResourceContents::text(
                            "This is example content from memory",
                            &request.uri
                        )
                    ],
                })
            }
            _ => Err(ErrorData {
                code: ErrorCode(-32601),
                message: format!("Resource '{}' not found", request.uri).into(),
                data: None,
            }),
        }
    }
}
