import { APIGatewayProxyHandler } from "aws-lambda";
import { mcpHandler } from "./handlers/mcp-handler";

export const handler: APIGatewayProxyHandler = async (event, context) => {
  return await mcpHandler(event, context);
};
