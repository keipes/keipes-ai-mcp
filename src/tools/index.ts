import { McpServer } from "../types/mcp-sdk";
import { calculatorToolSchema, calculatorToolCallback } from "./calculator";
import { weatherToolSchema, weatherToolCallback } from "./weather";

export const registerTools = (server: McpServer) => {
  server.tool(
    "calculator",
    "Performs basic mathematical calculations",
    calculatorToolSchema,
    calculatorToolCallback
  );

  server.tool(
    "weather",
    "Gets weather information for a location",
    weatherToolSchema,
    weatherToolCallback
  );
};
