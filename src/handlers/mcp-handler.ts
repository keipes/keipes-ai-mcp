import {
  APIGatewayProxyEvent,
  APIGatewayProxyResult,
  Context,
} from "aws-lambda";
import { logger } from "../utils/logger";
import {
  calculatorToolCallback,
  calculatorToolSchema,
} from "../tools/calculator";
import { weatherToolCallback, weatherToolSchema } from "../tools/weather";
import { z } from "zod";

// Simple MCP-compatible handler for serverless environment
export const mcpHandler = async (
  event: APIGatewayProxyEvent,
  context: Context
): Promise<APIGatewayProxyResult> => {
  try {
    const requestBody = JSON.parse(event.body || "{}");
    const { method, params } = requestBody;

    let result;

    switch (method) {
      case "tools/list":
        result = {
          tools: [
            {
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
            },
            {
              name: "weather",
              description: "Gets weather information for a location",
              inputSchema: {
                type: "object",
                properties: {
                  location: {
                    type: "string",
                    description: "The city or location to get weather for",
                  },
                  unit: {
                    type: "string",
                    enum: ["celsius", "fahrenheit"],
                    default: "celsius",
                  },
                },
                required: ["location"],
              },
            },
          ],
        };
        break;

      case "tools/call":
        const { name, arguments: args } = params;
        if (name === "calculator") {
          const parsed = z.object(calculatorToolSchema).parse(args);
          result = await calculatorToolCallback(parsed);
        } else if (name === "weather") {
          const parsed = z.object(weatherToolSchema).parse(args);
          result = await weatherToolCallback(parsed);
        } else {
          throw new Error(`Unknown tool: ${name}`);
        }
        break;

      case "initialize":
        result = {
          protocolVersion: "2025-06-18",
          capabilities: {
            tools: {},
          },
          serverInfo: {
            name: "keipes-ai-mcp",
            version: "1.0.0",
          },
        };
        break;

      default:
        throw new Error(`Unknown method: ${method}`);
    }

    return {
      statusCode: 200,
      headers: {
        "Content-Type": "application/json",
        "Access-Control-Allow-Origin": "*",
      },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: requestBody.id,
        result,
      }),
    };
  } catch (error) {
    logger.error("MCP handler error", { error });
    return {
      statusCode: 200,
      headers: {
        "Content-Type": "application/json",
        "Access-Control-Allow-Origin": "*",
      },
      body: JSON.stringify({
        jsonrpc: "2.0",
        id: JSON.parse(event.body || "{}").id,
        error: {
          code: -32603,
          message: "Internal error",
          data: error instanceof Error ? error.message : String(error),
        },
      }),
    };
  }
};
