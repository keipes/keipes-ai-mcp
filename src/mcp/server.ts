import { McpServer } from "../types/mcp-sdk";
import { registerTools } from "../tools";

export class MCPServer {
  private server: McpServer;

  constructor() {
    this.server = new McpServer(
      {
        name: "keipes-ai-mcp",
        version: "1.0.0",
      },
      {
        capabilities: {
          tools: {},
          resources: {},
        },
      }
    );

    this.setupTools();
  }

  private setupTools() {
    registerTools(this.server);
  }

  async connect(transport: any) {
    return await this.server.connect(transport);
  }

  async close() {
    return await this.server.close();
  }

  isConnected() {
    return this.server.isConnected();
  }
}
