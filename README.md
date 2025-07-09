# MCP Serverless Application

A serverless Model Context Protocol application built with Go and AWS CDK v2.

## Architecture

- **AWS Lambda**: Serverless compute for MCP server
- **API Gateway**: HTTP endpoints for MCP protocol
- **Go**: Primary programming language
- **CDK v2**: Infrastructure as Code

## Setup

1. Install dependencies:

   ```bash
   go mod download
   cd infrastructure && go mod download
   ```

2. Deploy infrastructure:

   ```bash
   ./scripts/deploy.sh
   ```

3. Test the deployment:
   ```bash
   ./scripts/test.sh
   ```

## Development

Run locally:

```bash
./scripts/local-dev.sh
```

## Project Structure

- `src/` - Core MCP server implementation
- `infrastructure/` - CDK infrastructure code
- `lambda/` - Lambda deployment artifacts
- `scripts/` - Development and deployment scripts
- `docs/` - Documentation
