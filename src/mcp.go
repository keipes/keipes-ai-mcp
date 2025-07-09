package main

import (
	"context"
	"encoding/json"

	"github.com/keipes-ai/mcp/src/handlers"
)

type McpServer struct {
	tools     map[string]Tool
	resources map[string]Resource
}

func NewMcpServer() *McpServer {
	return &McpServer{
		tools:     make(map[string]Tool),
		resources: make(map[string]Resource),
	}
}

func (s *McpServer) HandleRequest(ctx context.Context, request handlers.McpRequest) (handlers.McpResponse, error) {
	switch request.Method {
	case "initialize":
		return s.handleInitialize(request)
	case "tools/list":
		return s.handleToolsList(request)
	case "tools/call":
		return s.handleToolsCall(request)
	case "resources/list":
		return s.handleResourcesList(request)
	case "resources/read":
		return s.handleResourcesRead(request)
	default:
		return handlers.McpResponse{
			JsonRpc: "2.0",
			Id:      request.Id,
			Error: map[string]interface{}{
				"code":    -32601,
				"message": "Method not found",
			},
		}, nil
	}
}

func (s *McpServer) handleInitialize(req handlers.McpRequest) (handlers.McpResponse, error) {
	return handlers.McpResponse{
		JsonRpc: "2.0",
		Id:      req.Id,
		Result: map[string]interface{}{
			"protocolVersion": "2024-11-05",
			"capabilities": map[string]interface{}{
				"tools":     map[string]interface{}{},
				"resources": map[string]interface{}{},
			},
			"serverInfo": map[string]interface{}{
				"name":    "keipes-mcp-server",
				"version": "1.0.0",
			},
		},
	}, nil
}

func (s *McpServer) handleToolsList(req handlers.McpRequest) (handlers.McpResponse, error) {
	tools := make([]Tool, 0, len(s.tools))
	for _, tool := range s.tools {
		tools = append(tools, tool)
	}

	return handlers.McpResponse{
		JsonRpc: "2.0",
		Id:      req.Id,
		Result: map[string]interface{}{
			"tools": tools,
		},
	}, nil
}

func (s *McpServer) handleToolsCall(req handlers.McpRequest) (handlers.McpResponse, error) {
	var params struct {
		Name      string                 `json:"name"`
		Arguments map[string]interface{} `json:"arguments"`
	}

	if err := json.Unmarshal(req.Params, &params); err != nil {
		return handlers.McpResponse{
			JsonRpc: "2.0",
			Id:      req.Id,
			Error: map[string]interface{}{
				"code":    -32602,
				"message": "Invalid params",
			},
		}, nil
	}

	tool, exists := s.tools[params.Name]
	if !exists {
		return handlers.McpResponse{
			JsonRpc: "2.0",
			Id:      req.Id,
			Error: map[string]interface{}{
				"code":    -32601,
				"message": "Tool not found",
			},
		}, nil
	}

	result, err := tool.Execute(params.Arguments)
	if err != nil {
		return handlers.McpResponse{
			JsonRpc: "2.0",
			Id:      req.Id,
			Error: map[string]interface{}{
				"code":    -32603,
				"message": err.Error(),
			},
		}, nil
	}

	return handlers.McpResponse{
		JsonRpc: "2.0",
		Id:      req.Id,
		Result:  result,
	}, nil
}

func (s *McpServer) handleResourcesList(req handlers.McpRequest) (handlers.McpResponse, error) {
	resources := make([]Resource, 0, len(s.resources))
	for _, resource := range s.resources {
		resources = append(resources, resource)
	}

	return handlers.McpResponse{
		JsonRpc: "2.0",
		Id:      req.Id,
		Result: map[string]interface{}{
			"resources": resources,
		},
	}, nil
}

func (s *McpServer) handleResourcesRead(req handlers.McpRequest) (handlers.McpResponse, error) {
	var params struct {
		Uri string `json:"uri"`
	}

	if err := json.Unmarshal(req.Params, &params); err != nil {
		return handlers.McpResponse{
			JsonRpc: "2.0",
			Id:      req.Id,
			Error: map[string]interface{}{
				"code":    -32602,
				"message": "Invalid params",
			},
		}, nil
	}

	resource, exists := s.resources[params.Uri]
	if !exists {
		return handlers.McpResponse{
			JsonRpc: "2.0",
			Id:      req.Id,
			Error: map[string]interface{}{
				"code":    -32601,
				"message": "Resource not found",
			},
		}, nil
	}

	content, err := resource.Read()
	if err != nil {
		return handlers.McpResponse{
			JsonRpc: "2.0",
			Id:      req.Id,
			Error: map[string]interface{}{
				"code":    -32603,
				"message": err.Error(),
			},
		}, nil
	}

	return handlers.McpResponse{
		JsonRpc: "2.0",
		Id:      req.Id,
		Result: map[string]interface{}{
			"contents": []map[string]interface{}{
				{
					"uri":      params.Uri,
					"mimeType": resource.MimeType,
					"text":     content,
				},
			},
		},
	}, nil
}
