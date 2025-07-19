#!/bin/bash

TARGETARCH=arm64

# Stop and remove existing containers
docker ps -q --filter ancestor=keipes-ai-mcp | xargs -r docker stop
docker ps -aq --filter ancestor=keipes-ai-mcp | xargs -r docker rm
docker rm -f keipes-ai-mcp 2>/dev/null || true

# Build and run new container
if [ "$TARGETARCH" == "arm64" ]; then
    echo "Building for arm64."
    docker build --platform linux/arm64 -t keipes-ai-mcp:arm64 .
    docker run --platform linux/arm64 -d -p 80:80 --name keipes-ai-mcp keipes-ai-mcp:arm64
else
    echo "Building for default architecture."
    docker build -t keipes-ai-mcp .
    docker run -d -p 80:80 --name keipes-ai-mcp keipes-ai-mcp
fi

echo "Container started. To view logs, run:"
echo "docker logs -f keipes-ai-mcp"
echo ""
echo "To view the log file inside the container:"
echo "docker exec keipes-ai-mcp tail -f /var/log/keipes-ai-mcp.log"


# docker build --platform linux/arm64 --progress=plain -t keipes-ai-mcp:arm64 .