import * as cdk from "aws-cdk-lib";
import * as apigateway from "aws-cdk-lib/aws-apigateway";
import * as apigatewayv2 from "aws-cdk-lib/aws-apigatewayv2";
import { Construct } from "constructs";

export class MCPApiGateway extends Construct {
  public readonly restApi: apigateway.RestApi;
  public readonly webSocketApi: apigatewayv2.WebSocketApi;

  constructor(scope: Construct, id: string) {
    super(scope, id);

    this.restApi = new apigateway.RestApi(this, "RestAPI", {
      restApiName: "MCP REST API",
    });

    this.webSocketApi = new apigatewayv2.WebSocketApi(this, "WebSocketAPI", {
      apiName: "MCP WebSocket API",
    });
  }
}
