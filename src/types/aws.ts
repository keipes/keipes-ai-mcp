import {
  APIGatewayProxyEvent,
  APIGatewayProxyResult,
  Context,
} from "aws-lambda";

export type LambdaEvent = APIGatewayProxyEvent;
export type LambdaContext = Context;
export type APIGatewayEvent = APIGatewayProxyEvent;
