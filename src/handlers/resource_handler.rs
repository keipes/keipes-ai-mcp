use rmcp::model::*;
use std::sync::Arc;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

pub trait ResourceTrait {
    fn uri(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn mime_type(&self) -> Option<&str>;
    fn size(&self) -> Option<u32>;
    fn read(&self) -> Pin<Box<dyn Future<Output = Result<Vec<ResourceContents>, ErrorData>> + Send + '_>>;
}

pub struct ExampleResource;

impl ResourceTrait for ExampleResource {
    fn uri(&self) -> &str {
        "memory://example"
    }

    fn name(&self) -> &str {
        "Example Resource"
    }

    fn description(&self) -> Option<&str> {
        Some("An example in-memory resource")
    }

    fn mime_type(&self) -> Option<&str> {
        Some("text/plain")
    }

    fn size(&self) -> Option<u32> {
        None
    }

    fn read(&self) -> Pin<Box<dyn Future<Output = Result<Vec<ResourceContents>, ErrorData>> + Send + '_>> {
        let uri = self.uri().to_string();
        Box::pin(async move {
            Ok(vec![
                ResourceContents::text(
                    "This is example content from memory",
                    &uri
                )
            ])
        })
    }
}

pub struct ResourceHandler {
    resources: HashMap<String, Arc<dyn ResourceTrait + Send + Sync>>,
}

impl Clone for ResourceHandler {
    fn clone(&self) -> Self {
        Self {
            resources: self.resources.clone(),
        }
    }
}

impl ResourceHandler {
    pub fn new() -> Self {
        let mut resources: HashMap<String, Arc<dyn ResourceTrait + Send + Sync>> = HashMap::new();
        let example_resource: Arc<dyn ResourceTrait + Send + Sync> = Arc::new(ExampleResource);
        resources.insert(example_resource.uri().to_string(), example_resource);
        
        Self { resources }
    }

    pub fn capabilities(&self) -> HashMap<String, serde_json::Value> {
        self.resources.iter().map(|(uri, resource)| {
            (uri.clone(), serde_json::json!({
                "name": resource.name(),
                "description": resource.description(),
                "mimeType": resource.mime_type()
            }))
        }).collect()
    }

    pub async fn list_resources(&self, _request: Option<PaginatedRequestParam>) -> ListResourcesResult {
        use rmcp::model::RawResource;
        
        let resources = self.resources.values().map(|resource| {
            Resource::new(
                RawResource {
                    uri: resource.uri().into(),
                    name: resource.name().to_string(),
                    description: resource.description().map(|d| d.to_string()),
                    mime_type: resource.mime_type().map(|m| m.to_string()),
                    size: resource.size(),
                },
                None
            )
        }).collect();
        
        ListResourcesResult {
            resources,
            next_cursor: None,
        }
    }

    pub async fn read_resource(&self, request: ReadResourceRequestParam) -> Result<ReadResourceResult, ErrorData> {
        if let Some(resource) = self.resources.get(&request.uri) {
            let contents = resource.read().await?;
            Ok(ReadResourceResult { contents })
        } else {
            Err(ErrorData {
                code: ErrorCode(-32601),
                message: format!("Resource '{}' not found", request.uri).into(),
                data: None,
            })
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
