import { APIGatewayProxyEvent, APIGatewayProxyResult } from "aws-lambda";
import { getAllTools } from "../tools";

export const toolsHandler = async (
  event: APIGatewayProxyEvent
): Promise<APIGatewayProxyResult> => {
  const tools = getAllTools();

  return {
    statusCode: 200,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ tools }),
  };
};
