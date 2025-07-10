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

pub trait ResourceTemplateTrait {
    fn uri_template(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
    fn mime_type(&self) -> Option<&str>;
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

pub struct TemplateResource;

impl ResourceTrait for TemplateResource {
    fn uri(&self) -> &str {
        "template://greeting"
    }

    fn name(&self) -> &str {
        "Greeting Template"
    }

    fn description(&self) -> Option<&str> {
        Some("A template for greeting messages")
    }

    fn mime_type(&self) -> Option<&str> {
        Some("text/template")
    }

    fn size(&self) -> Option<u32> {
        Some(256)
    }

    fn read(&self) -> Pin<Box<dyn Future<Output = Result<Vec<ResourceContents>, ErrorData>> + Send + '_>> {
        let uri = self.uri().to_string();
        Box::pin(async move {
            let template_content = r#"
# Greeting Template

Hello, {{name}}!

Welcome to our {{service}}. We're excited to have you here.

## Next Steps
1. Explore our features
2. Configure your preferences
3. Start using {{service}}

Best regards,
The {{service}} Team
"#;
            Ok(vec![
                ResourceContents::text(template_content, &uri)
            ])
        })
    }
}

pub struct CodeTemplateResource;

impl ResourceTrait for CodeTemplateResource {
    fn uri(&self) -> &str {
        "template://rust-function"
    }

    fn name(&self) -> &str {
        "Rust Function Template"
    }

    fn description(&self) -> Option<&str> {
        Some("A template for creating Rust functions")
    }

    fn mime_type(&self) -> Option<&str> {
        Some("text/rust")
    }

    fn size(&self) -> Option<u32> {
        Some(512)
    }

    fn read(&self) -> Pin<Box<dyn Future<Output = Result<Vec<ResourceContents>, ErrorData>> + Send + '_>> {
        let uri = self.uri().to_string();
        Box::pin(async move {
            let template_content = r#"
/// {{description}}
/// 
/// # Arguments
/// 
/// * `{{param_name}}` - {{param_description}}
/// 
/// # Returns
/// 
/// {{return_description}}
/// 
/// # Example
/// 
/// ```
/// let result = {{function_name}}({{example_param}});
/// assert_eq!(result, {{expected_result}});
/// ```
pub fn {{function_name}}({{param_name}}: {{param_type}}) -> {{return_type}} {
    // TODO: Implement function logic
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_{{function_name}}() {
        // TODO: Add test cases
        todo!()
    }
}
"#;
            Ok(vec![
                ResourceContents::text(template_content, &uri)
            ])
        })
    }
}

pub struct ResourceHandler {
    resources: HashMap<String, Arc<dyn ResourceTrait + Send + Sync>>,
    resource_templates: HashMap<String, Arc<dyn ResourceTemplateTrait + Send + Sync>>,
}

impl Clone for ResourceHandler {
    fn clone(&self) -> Self {
        Self {
            resources: self.resources.clone(),
            resource_templates: self.resource_templates.clone(),
        }
    }
}

impl ResourceHandler {
    pub fn new() -> Self {
        let mut resources: HashMap<String, Arc<dyn ResourceTrait + Send + Sync>> = HashMap::new();
        
        // Add example resource
        let example_resource: Arc<dyn ResourceTrait + Send + Sync> = Arc::new(ExampleResource);
        resources.insert(example_resource.uri().to_string(), example_resource);
        
        // Add template resources
        let greeting_template: Arc<dyn ResourceTrait + Send + Sync> = Arc::new(TemplateResource);
        resources.insert(greeting_template.uri().to_string(), greeting_template);
        
        let code_template: Arc<dyn ResourceTrait + Send + Sync> = Arc::new(CodeTemplateResource);
        resources.insert(code_template.uri().to_string(), code_template);
        
        let mut resource_templates: HashMap<String, Arc<dyn ResourceTemplateTrait + Send + Sync>> = HashMap::new();
        
        // Add example resource template
        let example_resource_template: Arc<dyn ResourceTemplateTrait + Send + Sync> = Arc::new(ExampleResourceTemplate);
        resource_templates.insert(example_resource_template.uri_template().to_string(), example_resource_template);
        
        Self { resources, resource_templates }
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

    pub async fn list_resource_templates(&self, _request: Option<PaginatedRequestParam>) -> ListResourceTemplatesResult {
        use rmcp::model::RawResourceTemplate;
        
        let resource_templates = self.resource_templates.values().map(|template| {
            ResourceTemplate::new(
                RawResourceTemplate {
                    uri_template: template.uri_template().to_string(),
                    name: template.name().to_string(),
                    description: template.description().map(|d| d.to_string()),
                    mime_type: template.mime_type().map(|m| m.to_string()),
                },
                None
            )
        }).collect();
        
        ListResourceTemplatesResult {
            resource_templates,
            next_cursor: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_resource_properties() {
        let resource = ExampleResource;
        assert_eq!(resource.uri(), "memory://example");
        assert_eq!(resource.name(), "Example Resource");
        assert_eq!(resource.description(), Some("An example in-memory resource"));
        assert_eq!(resource.mime_type(), Some("text/plain"));
        assert_eq!(resource.size(), None);
    }

    #[tokio::test]
    async fn test_example_resource_read() {
        let resource = ExampleResource;
        let result = resource.read().await;
        assert!(result.is_ok());
        let contents = result.unwrap();
        assert_eq!(contents.len(), 1);
    }

    #[test]
    fn test_template_resource_properties() {
        let resource = TemplateResource;
        assert_eq!(resource.uri(), "template://greeting");
        assert_eq!(resource.name(), "Greeting Template");
        assert_eq!(resource.description(), Some("A template for greeting messages"));
        assert_eq!(resource.mime_type(), Some("text/template"));
        assert_eq!(resource.size(), Some(256));
    }

    #[tokio::test]
    async fn test_template_resource_read() {
        let resource = TemplateResource;
        let result = resource.read().await;
        assert!(result.is_ok());
        let contents = result.unwrap();
        assert_eq!(contents.len(), 1);
    }

    #[test]
    fn test_code_template_resource_properties() {
        let resource = CodeTemplateResource;
        assert_eq!(resource.uri(), "template://rust-function");
        assert_eq!(resource.name(), "Rust Function Template");
        assert_eq!(resource.description(), Some("A template for creating Rust functions"));
        assert_eq!(resource.mime_type(), Some("text/rust"));
        assert_eq!(resource.size(), Some(512));
    }

    #[tokio::test]
    async fn test_code_template_resource_read() {
        let resource = CodeTemplateResource;
        let result = resource.read().await;
        assert!(result.is_ok());
        let contents = result.unwrap();
        assert_eq!(contents.len(), 1);
    }

    #[test]
    fn test_example_resource_template_properties() {
        let template = ExampleResourceTemplate;
        assert_eq!(template.uri_template(), "memory://items/{id}");
        assert_eq!(template.name(), "Item Resource");
        assert_eq!(template.description(), Some("A template for accessing items by ID"));
        assert_eq!(template.mime_type(), Some("application/json"));
    }

    #[test]
    fn test_resource_handler_new() {
        let handler = ResourceHandler::new();
        assert_eq!(handler.resources.len(), 3);
        assert_eq!(handler.resource_templates.len(), 1);
    }

    #[test]
    fn test_resource_handler_capabilities() {
        let handler = ResourceHandler::new();
        let capabilities = handler.capabilities();
        assert_eq!(capabilities.len(), 3);
        assert!(capabilities.contains_key("memory://example"));
        assert!(capabilities.contains_key("template://greeting"));
        assert!(capabilities.contains_key("template://rust-function"));
    }

    #[test]
    fn test_resource_handler_clone() {
        let handler = ResourceHandler::new();
        let cloned = handler.clone();
        assert_eq!(handler.resources.len(), cloned.resources.len());
        assert_eq!(handler.resource_templates.len(), cloned.resource_templates.len());
    }

    #[tokio::test]
    async fn test_list_resources() {
        let handler = ResourceHandler::new();
        
        let result = handler.list_resources(None).await;
        
        assert_eq!(result.resources.len(), 3);
        
        // Find the example resource
        let example_resource = result.resources.iter()
            .find(|r| r.uri == "memory://example")
            .expect("Example resource should exist");
        
        assert_eq!(example_resource.name, "Example Resource");
        assert_eq!(example_resource.description, Some("An example in-memory resource".to_string()));
        assert_eq!(example_resource.mime_type, Some("text/plain".to_string()));
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

    #[tokio::test]
    async fn test_list_resource_templates() {
        let handler = ResourceHandler::new();
        
        let result = handler.list_resource_templates(None).await;
        
        assert_eq!(result.resource_templates.len(), 1);
        assert_eq!(result.resource_templates[0].raw.uri_template, "memory://items/{id}");
        assert_eq!(result.resource_templates[0].raw.name, "Item Resource");
        assert_eq!(result.resource_templates[0].raw.description, Some("A template for accessing items by ID".to_string()));
        assert_eq!(result.resource_templates[0].raw.mime_type, Some("application/json".to_string()));
        assert!(result.next_cursor.is_none());
    }
}

pub struct ExampleResourceTemplate;

impl ResourceTemplateTrait for ExampleResourceTemplate {
    fn uri_template(&self) -> &str {
        "memory://items/{id}"
    }

    fn name(&self) -> &str {
        "Item Resource"
    }

    fn description(&self) -> Option<&str> {
        Some("A template for accessing items by ID")
    }

    fn mime_type(&self) -> Option<&str> {
        Some("application/json")
    }
}
