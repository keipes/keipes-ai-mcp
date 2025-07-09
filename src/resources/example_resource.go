package resources

import "time"

type ExampleResource struct {
	uri         string
	name        string
	description string
	content     string
}

func NewExampleResource() *ExampleResource {
	return &ExampleResource{
		uri:         "example://resource/1",
		name:        "Example Resource",
		description: "An example MCP resource",
		content:     "This is example resource content",
	}
}

func (r *ExampleResource) Uri() string {
	return r.uri
}

func (r *ExampleResource) Name() string {
	return r.name
}

func (r *ExampleResource) Description() string {
	return r.description
}

func (r *ExampleResource) MimeType() string {
	return "text/plain"
}

func (r *ExampleResource) Read() (string, error) {
	timestamp := time.Now().Format(time.RFC3339)
	return r.content + "\n\nLast accessed: " + timestamp, nil
}
