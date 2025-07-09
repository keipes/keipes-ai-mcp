package main

// Tool represents an MCP tool
type Tool struct {
	Name        string                                                 `json:"name"`
	Description string                                                 `json:"description"`
	InputSchema map[string]interface{}                                 `json:"inputSchema"`
	Execute     func(args map[string]interface{}) (interface{}, error) `json:"-"`
}

// Resource represents an MCP resource
type Resource struct {
	Uri         string                 `json:"uri"`
	Name        string                 `json:"name"`
	Description string                 `json:"description,omitempty"`
	MimeType    string                 `json:"mimeType,omitempty"`
	Read        func() (string, error) `json:"-"`
}
