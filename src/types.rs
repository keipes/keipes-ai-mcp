use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: McpCapabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpCapabilities {
    pub tools: HashMap<String, serde_json::Value>,
    pub resources: HashMap<String, serde_json::Value>,
    pub prompts: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerDetails {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub tools: bool,
    pub resources: bool,
    pub prompts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_address: String,
    pub port: u16,
    pub sse_path: String,
    pub post_path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_info_serialization() {
        let server_info = ServerInfo {
            protocol_version: "2024-11-05".to_string(),
            capabilities: McpCapabilities {
                tools: HashMap::new(),
                resources: HashMap::new(),
                prompts: HashMap::new(),
            },
            server_info: ServerDetails {
                name: "test-server".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let serialized = serde_json::to_string(&server_info).unwrap();
        assert!(serialized.contains("protocolVersion"));
        assert!(serialized.contains("serverInfo"));
    }

    #[test]
    fn test_server_info_deserialization() {
        let json = r#"{
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {}
            },
            "serverInfo": {
                "name": "test-server",
                "version": "1.0.0"
            }
        }"#;

        let server_info: ServerInfo = serde_json::from_str(json).unwrap();
        assert_eq!(server_info.protocol_version, "2024-11-05");
        assert_eq!(server_info.server_info.name, "test-server");
        assert_eq!(server_info.server_info.version, "1.0.0");
    }

    #[test]
    fn test_mcp_capabilities_creation() {
        let mut tools = HashMap::new();
        tools.insert("echo".to_string(), serde_json::json!({"description": "Echo tool"}));
        
        let capabilities = McpCapabilities {
            tools,
            resources: HashMap::new(),
            prompts: HashMap::new(),
        };

        assert_eq!(capabilities.tools.len(), 1);
        assert!(capabilities.tools.contains_key("echo"));
        assert_eq!(capabilities.resources.len(), 0);
        assert_eq!(capabilities.prompts.len(), 0);
    }

    #[test]
    fn test_server_details_creation() {
        let details = ServerDetails {
            name: "my-server".to_string(),
            version: "2.0.0".to_string(),
        };

        assert_eq!(details.name, "my-server");
        assert_eq!(details.version, "2.0.0");
    }

    #[test]
    fn test_server_capabilities_creation() {
        let capabilities = ServerCapabilities {
            tools: true,
            resources: false,
            prompts: true,
        };

        assert!(capabilities.tools);
        assert!(!capabilities.resources);
        assert!(capabilities.prompts);
    }

    #[test]
    fn test_server_config_creation() {
        let config = ServerConfig {
            bind_address: "127.0.0.1".to_string(),
            port: 8080,
            sse_path: "/sse".to_string(),
            post_path: "/post".to_string(),
        };

        assert_eq!(config.bind_address, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert_eq!(config.sse_path, "/sse");
        assert_eq!(config.post_path, "/post");
    }

    #[test]
    fn test_server_config_clone() {
        let config = ServerConfig {
            bind_address: "localhost".to_string(),
            port: 3000,
            sse_path: "/events".to_string(),
            post_path: "/messages".to_string(),
        };

        let cloned = config.clone();
        assert_eq!(config.bind_address, cloned.bind_address);
        assert_eq!(config.port, cloned.port);
        assert_eq!(config.sse_path, cloned.sse_path);
        assert_eq!(config.post_path, cloned.post_path);
    }
}
