package main

import (
	"github.com/aws/aws-cdk-go/awscdk/v2"
	"github.com/aws/aws-cdk-go/awscdk/v2/awsapigateway"
	"github.com/aws/aws-cdk-go/awscdk/v2/awslambda"
	"github.com/aws/constructs-go/constructs/v10"
	"github.com/aws/jsii-runtime-go"
)

type McpStackProps struct {
	awscdk.StackProps
}

func NewMcpStack(scope constructs.Construct, id string, props *McpStackProps) awscdk.Stack {
	var sprops awscdk.StackProps
	if props != nil {
		sprops = props.StackProps
	}
	stack := awscdk.NewStack(scope, &id, &sprops)

	// Lambda function
	lambdaFn := awslambda.NewFunction(stack, jsii.String("McpHandler"), &awslambda.FunctionProps{
		Runtime: awslambda.Runtime_PROVIDED_AL2(),
		Code:    awslambda.Code_FromAsset(jsii.String("../lambda/bootstrap.zip"), nil),
		Handler: jsii.String("bootstrap"),
		Environment: &map[string]*string{
			"GO_ENV": jsii.String("production"),
		},
	})

	// API Gateway
	api := awsapigateway.NewRestApi(stack, jsii.String("McpApi"), &awsapigateway.RestApiProps{
		RestApiName: jsii.String("MCP Server API"),
		Description: jsii.String("Model Context Protocol server endpoints"),
	})

	// Lambda integration
	integration := awsapigateway.NewLambdaIntegration(lambdaFn, &awsapigateway.LambdaIntegrationOptions{
		RequestTemplates: &map[string]*string{
			"application/json": jsii.String("{ \"statusCode\": \"200\" }"),
		},
	})

	// Add routes
	api.Root().AddMethod(jsii.String("POST"), integration, nil)
	mcp := api.Root().AddResource(jsii.String("mcp"), nil)
	mcp.AddMethod(jsii.String("POST"), integration, nil)

	return stack
}
