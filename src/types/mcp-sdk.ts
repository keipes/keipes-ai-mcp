// Re-export actual MCP SDK types and classes
export {
  McpServer,
  type ToolCallback,
  type RegisteredTool,
} from "@modelcontextprotocol/sdk/server/mcp.js";
export { StdioServerTransport } from "@modelcontextprotocol/sdk/server/stdio.js";
export { z } from "zod";
