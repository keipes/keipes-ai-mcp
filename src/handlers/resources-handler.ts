import { APIGatewayProxyEvent, APIGatewayProxyResult } from 'aws-lambda';
import { getAllResources } from '../resources';

export const resourcesHandler = async (
  event: APIGatewayProxyEvent
): Promise<APIGatewayProxyResult> => {
  const resources = getAllResources();
  
  return {
    statusCode: 200,
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ resources })
  };
};
