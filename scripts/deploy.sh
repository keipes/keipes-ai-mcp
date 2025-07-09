#!/bin/bash

# Deployment script for CDK stack

set -e

echo "Deploying MCP Serverless Application..."

# Build Lambda binary
echo "Building Lambda function..."
cd lambda && ./build.sh && cd ..

# Deploy infrastructure
echo "Deploying CDK stack..."
cd infrastructure

# Install dependencies if needed
if [ ! -f "go.sum" ]; then
    echo "Installing Go dependencies..."
    go mod download
fi

# Deploy stack
cdk deploy --require-approval never

echo "Deployment completed successfully!"
echo "Check AWS Console for API Gateway endpoint URL"
