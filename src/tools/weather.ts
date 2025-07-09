import axios from "axios";
import { MCPTool } from "../types/mcp-sdk";

export const weatherTool: MCPTool = {
  name: "weather",
  description: "Gets weather information for a location",
  inputSchema: {
    type: "object",
    properties: {
      location: { type: "string" },
    },
    required: ["location"],
  },
  handler: async (args: any) => {
    const { location } = args;

    // Mock weather data for demonstration
    return {
      location,
      temperature: "22°C",
      condition: "Sunny",
      humidity: "65%",
    };
  },
};
