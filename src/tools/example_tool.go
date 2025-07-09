package tools

import "fmt"

type ExampleTool struct{}

func NewExampleTool() *ExampleTool {
	return &ExampleTool{}
}

func (t *ExampleTool) Name() string {
	return "example_tool"
}

func (t *ExampleTool) Description() string {
	return "An example MCP tool that echoes input"
}

func (t *ExampleTool) InputSchema() map[string]interface{} {
	return map[string]interface{}{
		"type": "object",
		"properties": map[string]interface{}{
			"message": map[string]interface{}{
				"type":        "string",
				"description": "Message to echo back",
			},
		},
		"required": []string{"message"},
	}
}

func (t *ExampleTool) Execute(args map[string]interface{}) (interface{}, error) {
	message, ok := args["message"].(string)
	if !ok {
		return nil, fmt.Errorf("message parameter is required and must be a string")
	}

	return map[string]interface{}{
		"echo":      message,
		"tool_name": t.Name(),
		"timestamp": "2024-01-01T00:00:00Z",
	}, nil
}
