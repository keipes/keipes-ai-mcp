import {
  Server,
  CallToolRequestSchema,
  ListToolsRequestSchema,
} from "../types/mcp-sdk";
import { getAllTools } from "../tools";
import { getAllResources } from "../resources";

export class MCPServer {
  private server: Server;

  constructor() {
    this.server = new Server(
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

    this.setupHandlers();
  }

  private setupHandlers() {
    this.server.setRequestHandler(ListToolsRequestSchema, async () => {
      return { tools: getAllTools() };
    });

    this.server.setRequestHandler(
      CallToolRequestSchema,
      async (request: any) => {
        const tools = getAllTools();
        const tool = tools.find((t: any) => t.name === request.params.name);

        if (!tool) {
          throw new Error(`Tool ${request.params.name} not found`);
        }

        return tool.handler(request.params.arguments);
      }
    );
  }

  async handleRequest(requestBody: string) {
    const request = JSON.parse(requestBody);
    return await this.server.handleRequest(request);
  }
}
