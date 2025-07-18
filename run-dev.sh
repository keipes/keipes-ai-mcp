#!/bin/bash

# Stop and remove existing containers
docker ps -q --filter ancestor=keipes-ai-mcp | xargs -r docker stop
docker ps -aq --filter ancestor=keipes-ai-mcp | xargs -r docker rm
docker rm -f keipes-ai-mcp 2>/dev/null || true

# Build and run new container
docker build -t keipes-ai-mcp . && docker run -d -p 8000:80 --name keipes-ai-mcp keipes-ai-mcp

echo "Container started. To view logs, run:"
echo "docker logs -f keipes-ai-mcp"
echo ""
echo "To view the log file inside the container:"
echo "docker exec keipes-ai-mcp tail -f /var/log/keipes-ai-mcp.log"
