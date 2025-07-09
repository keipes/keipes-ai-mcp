import * as cdk from "aws-cdk-lib";
import * as lambda from "aws-cdk-lib/aws-lambda";
import { Construct } from "constructs";

interface LambdaFunctionProps {
  functionName: string;
  codePath: string;
  handler: string;
}

export class LambdaFunction extends Construct {
  public readonly function: lambda.Function;

  constructor(scope: Construct, id: string, props: LambdaFunctionProps) {
    super(scope, id);

    this.function = new lambda.Function(this, "Function", {
      functionName: props.functionName,
      runtime: lambda.Runtime.NODEJS_18_X,
      code: lambda.Code.fromAsset(props.codePath),
      handler: props.handler,
      timeout: cdk.Duration.seconds(30),
      memorySize: 256,
    });
  }
}
