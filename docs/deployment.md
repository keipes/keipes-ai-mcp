# Deployment Guide

## Prerequisites

- Go 1.21+
- AWS CLI configured
- AWS CDK v2 installed
- Node.js (for CDK)

## Local Development

1. Clone the repository
2. Install dependencies:

   ```bash
   go mod download
   cd infrastructure && go mod download
   ```

3. Run locally:

   ```bash
   ./scripts/local-dev.sh
   ```

4. Test the server:
   ```bash
   ./scripts/test.sh http://localhost:8080
   ```

## AWS Deployment

### Setup

1. Configure AWS credentials:

   ```bash
   aws configure
   ```

2. Bootstrap CDK (first time only):
   ```bash
   cd infrastructure
   cdk bootstrap
   ```

### Deploy

1. Deploy the stack:

   ```bash
   ./scripts/deploy.sh
   ```

2. Get the API Gateway URL from AWS Console or CDK output

3. Test the deployed API:
   ```bash
   ./scripts/test.sh https://your-api-gateway-url.amazonaws.com
   ```

## Environment Variables

### Lambda Environment

- `GO_ENV`: Set to "production" for production deployment

### CDK Environment

- `CDK_DEFAULT_ACCOUNT`: AWS account ID
- `CDK_DEFAULT_REGION`: AWS region

## Troubleshooting

### Build Issues

- Ensure Go 1.21+ is installed
- Check GOOS and GOARCH are set correctly for Lambda

### Deployment Issues

- Verify AWS credentials
- Check CDK bootstrap is complete
- Ensure proper IAM permissions

### Runtime Issues

- Check CloudWatch logs for Lambda errors
- Verify API Gateway configuration
- Test with curl or Postman
