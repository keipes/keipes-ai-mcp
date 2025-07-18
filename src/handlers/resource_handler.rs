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
        
        // Add weapons.json file resource
        let weapons_resource: Arc<dyn ResourceTrait + Send + Sync> = Arc::new(FileResource::new(
            "file://weapons".to_string(),
            "Weapons Database".to_string(),
            r"D:\code\resources\weapons.json".to_string(),
            Some("application/json".to_string()),
            Some("External weapons database JSON file".to_string())
        ));
        resources.insert(weapons_resource.uri().to_string(), weapons_resource);
        
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
        let request_uri = request.uri.trim_end_matches('/');
        if let Some(resource) = self.resources.get(request_uri) {
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

pub struct FileResource {
    uri: String,
    name: String,
    description: Option<String>,
    file_path: String,
    mime_type: Option<String>,
}

impl FileResource {
    pub fn new(uri: String, name: String, file_path: String, mime_type: Option<String>, description: Option<String>) -> Self {
        Self {
            uri,
            name,
            description,
            file_path,
            mime_type,
        }
    }
}

impl ResourceTrait for FileResource {
    fn uri(&self) -> &str {
        &self.uri
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    fn mime_type(&self) -> Option<&str> {
        self.mime_type.as_deref()
    }

    fn size(&self) -> Option<u32> {
        None
    }

    fn read(&self) -> Pin<Box<dyn Future<Output = Result<Vec<ResourceContents>, ErrorData>> + Send + '_>> {
        let uri = self.uri.clone();
        let file_path = self.file_path.clone();
        Box::pin(async move {
            match std::fs::read_to_string(&file_path) {
                Ok(content) => Ok(vec![ResourceContents::text(&content, &uri)]),
                Err(e) => Err(ErrorData {
                    code: ErrorCode(-32603),
                    message: format!("Failed to read file '{}': {}", file_path, e).into(),
                    data: None,
                })
            }
        })
    }
}
