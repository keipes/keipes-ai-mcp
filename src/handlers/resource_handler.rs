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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_resources() {
        let handler = ResourceHandler::new();
        
        let result = handler.list_resources(None).await;
        
        assert_eq!(result.resources.len(), 1);
        assert_eq!(result.resources[0].uri, "memory://example");
        assert_eq!(result.resources[0].name, "Example Resource");
        assert_eq!(result.resources[0].description, Some("An example in-memory resource".to_string()));
        assert_eq!(result.resources[0].mime_type, Some("text/plain".to_string()));
        assert!(result.next_cursor.is_none());
    }

    #[tokio::test]
    async fn test_read_existing_resource() {
        let handler = ResourceHandler::new();
        
        let request = ReadResourceRequestParam {
            uri: "memory://example".to_string(),
        };
        
        let result = handler.read_resource(request).await;
        
        assert!(result.is_ok());
        let read_result = result.unwrap();
        assert_eq!(read_result.contents.len(), 1);
        
        // Just verify that we have content - the exact enum structure varies by rmcp version
        assert!(!read_result.contents.is_empty());
    }

    #[tokio::test]
    async fn test_read_nonexistent_resource() {
        let handler = ResourceHandler::new();
        
        let request = ReadResourceRequestParam {
            uri: "memory://nonexistent".to_string(),
        };
        
        let result = handler.read_resource(request).await;
        
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(error.code, ErrorCode(-32601));
        assert!(error.message.contains("Resource 'memory://nonexistent' not found"));
    }
}
