#!/bin/bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

case "${1:-help}" in
    "start"|"up")
        echo "Starting keipes-ai-mcp with PostgreSQL..."
        docker-compose up -d
        echo "Services started. MCP server available at http://localhost:8000/mcp"
        ;;
    "stop"|"down")
        echo "Stopping services..."
        docker-compose down
        ;;
    "restart")
        echo "Restarting services..."
        docker-compose down
        docker-compose up -d
        ;;
    "logs")
        docker-compose logs -f
        ;;
    "build")
        echo "Building keipes-ai-mcp image..."
        docker-compose build
        ;;
    "test")
        echo "Testing MCP server connection..."
        npx -p mcp-remote@latest mcp-remote-client http://localhost:8000/mcp
        ;;
    "db")
        echo "Connecting to PostgreSQL..."
        docker-compose exec postgres psql -U keipes -d keipes_mcp
        ;;
    "clean")
        echo "Stopping and removing all containers and volumes..."
        docker-compose down -v
        docker-compose rm -f
        ;;
    "help"|*)
        echo "Usage: $0 {start|stop|restart|logs|build|test|db|clean}"
        echo ""
        echo "Commands:"
        echo "  start   - Start all services"
        echo "  stop    - Stop all services"
        echo "  restart - Restart all services"
        echo "  logs    - Follow service logs"
        echo "  build   - Build the MCP server image"
        echo "  test    - Test MCP server connection"
        echo "  db      - Connect to PostgreSQL"
        echo "  clean   - Stop and remove everything"
        ;;
esac
