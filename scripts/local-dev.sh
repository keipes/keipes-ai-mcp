#!/bin/bash

# Local development server startup

set -e

echo "Starting MCP server in development mode..."

# Check if Go is installed
if ! command -v go &> /dev/null; then
    echo "Go is not installed. Please install Go first."
    exit 1
fi

# Install dependencies
echo "Installing dependencies..."
go mod download

# Run the server
echo "Starting server on http://localhost:8080"
echo "Use Ctrl+C to stop the server"
echo ""

cd src && go run main.go
