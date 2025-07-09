package handlers

import (
	"context"
	"encoding/json"
)

// McpHandler interface for handling MCP requests
type McpHandler interface {
	HandleRequest(ctx context.Context, req McpRequest) (McpResponse, error)
}

// McpRequest represents an MCP JSON-RPC request
type McpRequest struct {
	JsonRpc string          `json:"jsonrpc"`
	Id      interface{}     `json:"id"`
	Method  string          `json:"method"`
	Params  json.RawMessage `json:"params,omitempty"`
}

// McpResponse represents an MCP JSON-RPC response
type McpResponse struct {
	JsonRpc string      `json:"jsonrpc"`
	Id      interface{} `json:"id"`
	Result  interface{} `json:"result,omitempty"`
	Error   interface{} `json:"error,omitempty"`
}

// LambdaEvent represents a basic Lambda event
type LambdaEvent struct {
	Body string `json:"body"`
}

// LambdaResponse represents a basic Lambda response
type LambdaResponse struct {
	StatusCode int               `json:"statusCode"`
	Body       string            `json:"body"`
	Headers    map[string]string `json:"headers"`
}

func Handler(mcpServer McpHandler) func(context.Context, LambdaEvent) (LambdaResponse, error) {
	return func(ctx context.Context, request LambdaEvent) (LambdaResponse, error) {
		var mcpReq McpRequest
		if err := json.Unmarshal([]byte(request.Body), &mcpReq); err != nil {
			return LambdaResponse{
				StatusCode: 400,
				Body:       `{"error": "Invalid JSON"}`,
				Headers: map[string]string{
					"Content-Type": "application/json",
				},
			}, nil
		}

		mcpResp, err := mcpServer.HandleRequest(ctx, mcpReq)
		if err != nil {
			return LambdaResponse{
				StatusCode: 500,
				Body:       `{"error": "Internal server error"}`,
				Headers: map[string]string{
					"Content-Type": "application/json",
				},
			}, nil
		}

		respBody, err := json.Marshal(mcpResp)
		if err != nil {
			return LambdaResponse{
				StatusCode: 500,
				Body:       `{"error": "Response marshaling error"}`,
				Headers: map[string]string{
					"Content-Type": "application/json",
				},
			}, nil
		}

		return LambdaResponse{
			StatusCode: 200,
			Body:       string(respBody),
			Headers: map[string]string{
				"Content-Type": "application/json",
			},
		}, nil
	}
}
