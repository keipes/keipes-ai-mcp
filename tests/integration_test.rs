use keipes_ai_mcp::McpServer;

#[tokio::test]
async fn test_server_startup() {
    let config = keipes_ai_mcp::types::ServerConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 8001,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
    };

    let server = McpServer::new(config);
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.start().await
    });
    
    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Send test JSON-RPC message
    let client = reqwest::Client::new();
    let test_message = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "tools/list",
        "id": 1
    });
    
    let response = client
        .post("http://127.0.0.1:8001/mcp")
        .header("Content-Type", "application/json")
        .json(&test_message)
        .send()
        .await;
        
    match response {
        Ok(resp) => {
            println!("Response status: {}", resp.status());
            let text = resp.text().await.unwrap_or_default();
            println!("Response body: {}", text);
            
            // Parse response and verify it contains the echo tool
            let json: serde_json::Value = serde_json::from_str(&text).expect("Valid JSON");
            assert_eq!(json["jsonrpc"], "2.0");
            assert_eq!(json["id"], 1);
            assert!(json["result"]["tools"].is_array());
            
            let tools = json["result"]["tools"].as_array().unwrap();
            assert_eq!(tools.len(), 1);
            assert_eq!(tools[0]["name"], "echo");
            
            println!("✓ Server correctly responded with tools list");
        }
        Err(e) => {
            panic!("Server should have responded but got error: {}", e);
        }
    }
    
    // Clean up
    server_handle.abort();
}

#[tokio::test]
async fn test_list_resources() {
    let config = keipes_ai_mcp::types::ServerConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 8002,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
    };

    let server = McpServer::new(config);
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.start().await
    });
    
    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Send resources/list JSON-RPC message
    let client = reqwest::Client::new();
    let test_message = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "resources/list",
        "id": 2
    });
    
    let response = client
        .post("http://127.0.0.1:8002/mcp")
        .header("Content-Type", "application/json")
        .json(&test_message)
        .send()
        .await;
        
    match response {
        Ok(resp) => {
            println!("Response status: {}", resp.status());
            let text = resp.text().await.unwrap_or_default();
            println!("Response body: {}", text);
            
            // Parse response and verify it contains the example resource
            let json: serde_json::Value = serde_json::from_str(&text).expect("Valid JSON");
            assert_eq!(json["jsonrpc"], "2.0");
            assert_eq!(json["id"], 2);
            assert!(json["result"]["resources"].is_array());
            
            let resources = json["result"]["resources"].as_array().unwrap();
            assert_eq!(resources.len(), 1);
            assert_eq!(resources[0]["uri"], "memory://example");
            assert_eq!(resources[0]["name"], "Example Resource");
            assert_eq!(resources[0]["description"], "An example in-memory resource");
            assert_eq!(resources[0]["mimeType"], "text/plain");
            
            println!("✓ Server correctly responded with resources list");
        }
        Err(e) => {
            panic!("Server should have responded but got error: {}", e);
        }
    }
    
    // Clean up
    server_handle.abort();
}

#[tokio::test]
async fn test_list_prompts() {
    let config = keipes_ai_mcp::types::ServerConfig {
        bind_address: "127.0.0.1".to_string(),
        port: 8003,
        sse_path: "/sse".to_string(),
        post_path: "/message".to_string(),
    };

    let server = McpServer::new(config);
    
    // Start server in background
    let server_handle = tokio::spawn(async move {
        server.start().await
    });
    
    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Send prompts/list JSON-RPC message
    let client = reqwest::Client::new();
    let test_message = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "prompts/list",
        "id": 3
    });
    
    let response = client
        .post("http://127.0.0.1:8003/mcp")
        .header("Content-Type", "application/json")
        .json(&test_message)
        .send()
        .await;
        
    match response {
        Ok(resp) => {
            println!("Response status: {}", resp.status());
            let text = resp.text().await.unwrap_or_default();
            println!("Response body: {}", text);
            
            // Parse response and verify it contains the greeting prompt
            let json: serde_json::Value = serde_json::from_str(&text).expect("Valid JSON");
            assert_eq!(json["jsonrpc"], "2.0");
            assert_eq!(json["id"], 3);
            assert!(json["result"]["prompts"].is_array());
            
            let prompts = json["result"]["prompts"].as_array().unwrap();
            assert_eq!(prompts.len(), 1);
            assert_eq!(prompts[0]["name"], "greeting");
            assert_eq!(prompts[0]["description"], "A simple greeting prompt");
            assert!(prompts[0]["arguments"].is_array());
            
            let args = prompts[0]["arguments"].as_array().unwrap();
            assert_eq!(args.len(), 1);
            assert_eq!(args[0]["name"], "name");
            assert_eq!(args[0]["required"], true);
            
            println!("✓ Server correctly responded with prompts list");
        }
        Err(e) => {
            panic!("Server should have responded but got error: {}", e);
        }
    }
    
    // Clean up
    server_handle.abort();
}
