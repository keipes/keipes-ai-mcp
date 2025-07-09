# MCP API Documentation

## Model Context Protocol Server

This server implements the Model Context Protocol (MCP) specification for providing tools and resources to AI applications.

## Endpoints

### Base URL

- Local: `http://localhost:8080`
- Production: `https://your-api-gateway-url.amazonaws.com`

### JSON-RPC Methods

#### Initialize

Initialize the MCP connection.

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "initialize",
  "params": {
    "protocolVersion": "2024-11-05",
    "capabilities": {},
    "clientInfo": {
      "name": "client-name",
      "version": "1.0.0"
    }
  }
}
```

#### Tools List

Get available tools.

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "tools/list"
}
```

#### Tools Call

Execute a tool.

```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "tools/call",
  "params": {
    "name": "example_tool",
    "arguments": {
      "message": "Hello, World!"
    }
  }
}
```

#### Resources List

Get available resources.

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "resources/list"
}
```

#### Resources Read

Read a resource.

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "resources/read",
  "params": {
    "uri": "example://resource/1"
  }
}
```

## Error Codes

- `-32601`: Method not found
- `-32602`: Invalid params
- `-32603`: Internal error
