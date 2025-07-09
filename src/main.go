package main

import (
	"log"
	"net/http"

	"github.com/keipes-ai/mcp/src/handlers"
)

func main() {
	server := NewMcpServer()

	http.HandleFunc("/", handlers.HttpHandler(server))
	http.HandleFunc("/mcp", handlers.HttpHandler(server))

	log.Println("Starting MCP server on :8080")
	log.Fatal(http.ListenAndServe(":8080", nil))
}
