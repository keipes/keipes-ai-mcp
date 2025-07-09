package resources

import "sync"

type Resource interface {
	Uri() string
	Name() string
	Description() string
	MimeType() string
	Read() (string, error)
}

type ResourceRegistry struct {
	resources map[string]Resource
	mutex     sync.RWMutex
}

func NewResourceRegistry() *ResourceRegistry {
	return &ResourceRegistry{
		resources: make(map[string]Resource),
	}
}

func (r *ResourceRegistry) RegisterResource(resource Resource) {
	r.mutex.Lock()
	defer r.mutex.Unlock()
	r.resources[resource.Uri()] = resource
}

func (r *ResourceRegistry) GetResource(uri string) (Resource, bool) {
	r.mutex.RLock()
	defer r.mutex.RUnlock()
	resource, exists := r.resources[uri]
	return resource, exists
}

func (r *ResourceRegistry) ListResources() []Resource {
	r.mutex.RLock()
	defer r.mutex.RUnlock()

	resources := make([]Resource, 0, len(r.resources))
	for _, resource := range r.resources {
		resources = append(resources, resource)
	}
	return resources
}
