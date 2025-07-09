package tools

import "sync"

type Tool interface {
	Name() string
	Description() string
	InputSchema() map[string]interface{}
	Execute(args map[string]interface{}) (interface{}, error)
}

type ToolRegistry struct {
	tools map[string]Tool
	mutex sync.RWMutex
}

func NewToolRegistry() *ToolRegistry {
	return &ToolRegistry{
		tools: make(map[string]Tool),
	}
}

func (r *ToolRegistry) RegisterTool(tool Tool) {
	r.mutex.Lock()
	defer r.mutex.Unlock()
	r.tools[tool.Name()] = tool
}

func (r *ToolRegistry) GetTool(name string) (Tool, bool) {
	r.mutex.RLock()
	defer r.mutex.RUnlock()
	tool, exists := r.tools[name]
	return tool, exists
}

func (r *ToolRegistry) ListTools() []Tool {
	r.mutex.RLock()
	defer r.mutex.RUnlock()

	tools := make([]Tool, 0, len(r.tools))
	for _, tool := range r.tools {
		tools = append(tools, tool)
	}
	return tools
}
