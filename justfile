# Justfile for Keipes AI MCP
# cargo install just

# Variables
project := "keipes-ai-mcp"
port := "80"

# Build the Docker image
build:
    docker build -t {{project}} .

# Run the Docker container
run:
    docker run -d -p {{port}}:{{port}} --name {{project}} {{project}}

# Stop and remove the Docker container
stop:
    docker stop {{project}}; docker rm {{project}}

# Rebuild and restart the container
restart:
    just stop; just build; just run
