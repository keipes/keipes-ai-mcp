import * as cdk from "aws-cdk-lib";
import * as lambda from "aws-cdk-lib/aws-lambda";
import * as apigateway from "aws-cdk-lib/aws-apigateway";
import { Construct } from "constructs";
import { LambdaFunction } from "../constructs/lambda-function";

export class APIStack extends cdk.NestedStack {
  constructor(scope: Construct, id: string, props?: cdk.NestedStackProps) {
    super(scope, id, props);

    const mcpLambda = new LambdaFunction(this, "MCPLambda", {
      functionName: "mcp-handler",
      codePath: "./dist/src",
      handler: "index.handler",
    });

    new apigateway.RestApi(this, "MCPAPI", {
      restApiName: "MCP Service",
      defaultIntegration: new apigateway.LambdaIntegration(mcpLambda.function),
    });
  }
}
