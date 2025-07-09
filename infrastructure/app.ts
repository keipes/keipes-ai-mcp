#!/usr/bin/env node
import * as cdk from "aws-cdk-lib";
import { MCPStack } from "./stacks/mcp-stack.js";

const app = new cdk.App();

new MCPStack(app, "MCPStack", {
  env: {
    account: process.env.CDK_DEFAULT_ACCOUNT,
    region: process.env.CDK_DEFAULT_REGION,
  },
});
