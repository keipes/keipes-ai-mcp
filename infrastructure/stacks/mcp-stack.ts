import * as cdk from "aws-cdk-lib";
import { Construct } from "constructs";
import { APIStack } from "./api-stack.js";
import { StorageStack } from "./storage-stack.js";

export class MCPStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const storageStack = new StorageStack(this, "Storage");
    const apiStack = new APIStack(this, "API");
  }
}
