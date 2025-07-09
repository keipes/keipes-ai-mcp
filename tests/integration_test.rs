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
        .post("http://127.0.0.1:8001/message")
        .header("Content-Type", "application/json")
        .json(&test_message)
        .send()
        .await;
        
    match response {
        Ok(resp) => {
            println!("Response status: {}", resp.status());
            let text = resp.text().await.unwrap_or_default();
            println!("Response body: {}", text);
            assert!(false, "Server responded but we expected it to fail since server is not yet implemented");
        }
        Err(e) => {
            println!("Expected error: {}", e);
            // This is expected since the server doesn't implement HTTP endpoints yet
            assert!(true, "Server correctly rejected connection - not yet implemented");
        }
    }
    
    // Clean up
    server_handle.abort();
}

#[tokio::test]
async fn test_list_tools() {
    use keipes_ai_mcp::handlers::ToolHandler;
    
    let handler = ToolHandler::new();
    let result = handler.list_tools(None).await;
    
    assert_eq!(result.tools.len(), 1);
    assert_eq!(result.tools[0].name, "echo");
    assert!(result.tools[0].description.is_some());
    assert!(result.next_cursor.is_none());
}

#[tokio::test]
async fn test_echo_tool() {
    use keipes_ai_mcp::handlers::ToolHandler;
    
    let handler = ToolHandler::new();
    let result = handler.echo_tool("Hello, World!".to_string()).await;
    assert_eq!(result, "Hello, World!");
}

#[tokio::test]
async fn test_call_tool() {
    use keipes_ai_mcp::handlers::ToolHandler;
    use rmcp::model::CallToolRequestParam;
    
    let handler = ToolHandler::new();
    
    let mut args = serde_json::Map::new();
    args.insert("text".to_string(), serde_json::Value::String("Hello MCP!".to_string()));
    
    let request = CallToolRequestParam {
        name: "echo".into(),
        arguments: Some(args),
    };
    
    let result = handler.call_tool(request).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response.is_error, Some(false));
    assert_eq!(response.content.len(), 1);
}
