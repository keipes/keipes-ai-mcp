import { MCPTool } from "../types/mcp-sdk";

export const calculatorTool: MCPTool = {
  name: "calculator",
  description: "Performs basic mathematical calculations",
  inputSchema: {
    type: "object",
    properties: {
      operation: {
        type: "string",
        enum: ["add", "subtract", "multiply", "divide"],
      },
      a: { type: "number" },
      b: { type: "number" },
    },
    required: ["operation", "a", "b"],
  },
  handler: async (args: any) => {
    const { operation, a, b } = args;

    switch (operation) {
      case "add":
        return { result: a + b };
      case "subtract":
        return { result: a - b };
      case "multiply":
        return { result: a * b };
      case "divide":
        return { result: b !== 0 ? a / b : "Error: Division by zero" };
      default:
        throw new Error("Unknown operation");
    }
  },
};
