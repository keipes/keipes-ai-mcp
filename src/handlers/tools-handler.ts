import { APIGatewayProxyEvent, APIGatewayProxyResult } from "aws-lambda";

export const toolsHandler = async (
  event: APIGatewayProxyEvent
): Promise<APIGatewayProxyResult> => {
  const tools = [
    {
      name: "calculator",
      description: "Performs basic mathematical calculations",
    },
    {
      name: "weather",
      description: "Gets weather information for a location",
    },
  ];

  return {
    statusCode: 200,
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ tools }),
  };
};
