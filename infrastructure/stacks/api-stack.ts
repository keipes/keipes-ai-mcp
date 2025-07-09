import * as cdk from "aws-cdk-lib";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { Construct } from "constructs";
import { LambdaFunction } from "../constructs/lambda-function.js";

export class APIStack extends cdk.NestedStack {
  constructor(scope: Construct, id: string, props?: cdk.NestedStackProps) {
    super(scope, id, props);

    const mcpLambda = new LambdaFunction(this, "MCPLambda", {
      functionName: "mcp-handler",
      codePath: "./dist/src",
      handler: "index.handler",
    });

    const api = new apigateway.RestApi(this, "MCPAPI", {
      restApiName: "MCP Service",
      description: "Model Context Protocol API",
    });

    // Add a basic route for MCP requests
    const mcpResource = api.root.addResource("mcp");
    mcpResource.addMethod(
      "POST",
      new apigateway.LambdaIntegration(mcpLambda.function)
    );

    // Add health check endpoint
    const healthResource = api.root.addResource("health");
    healthResource.addMethod(
      "GET",
      new apigateway.LambdaIntegration(mcpLambda.function)
    );
  }
}
