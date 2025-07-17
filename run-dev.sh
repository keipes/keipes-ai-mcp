#!/bin/bash
docker ps -q --filter ancestor=keipes-ai-mcp | xargs -r docker stop && docker build -t keipes-ai-mcp . && docker run -d -p 8000:80 keipes-ai-mcp
