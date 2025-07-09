// Mock MCP SDK interfaces for development
export interface MCPTool {
  name: string;
  description: string;
  inputSchema: any;
  handler: (args: any) => Promise<any>;
}

export interface MCPResource {
  name: string;
  description: string;
  readFile?: (path: string) => Promise<string>;
  listDirectory?: (path: string) => Promise<string[]>;
  fetchContent?: (url: string) => Promise<string>;
}

export class Server {
  constructor(
    public config: { name: string; version: string },
    public capabilities: { capabilities: any }
  ) {}

  setRequestHandler(schema: any, handler: (request?: any) => Promise<any>) {
    // Mock implementation
  }

  async handleRequest(request: any) {
    return { result: "mock" };
  }
}

export const ListToolsRequestSchema = {};
export const CallToolRequestSchema = {};
