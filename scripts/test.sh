#!/bin/bash

# Test script for MCP endpoints

set -e

API_ENDPOINT=${1:-"http://localhost:8080"}

echo "Testing MCP server at: $API_ENDPOINT"

# Test initialize method
echo "Testing initialize..."
curl -X POST "$API_ENDPOINT/mcp" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
      "protocolVersion": "2024-11-05",
      "capabilities": {},
      "clientInfo": {
        "name": "test-client",
        "version": "1.0.0"
      }
    }
  }' | jq .

echo -e "\n"

# Test tools/list method
echo "Testing tools/list..."
curl -X POST "$API_ENDPOINT/mcp" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/list"
  }' | jq .

echo -e "\n"

# Test resources/list method
echo "Testing resources/list..."
curl -X POST "$API_ENDPOINT/mcp" \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 3,
    "method": "resources/list"
  }' | jq .

echo "Testing completed!"
