# Keipes AI MCP

A serverless Model Context Protocol (MCP) application built with AWS CDK v2 and TypeScript.

## Architecture

This project implements a serverless MCP server using:

- AWS Lambda for compute
- API Gateway for HTTP/WebSocket communication
- AWS CDK v2 for infrastructure as code
- TypeScript as the primary language

## Quick Start

1. Install dependencies:

```bash
npm install
```

2. Build the project:

```bash
npm run build
```

3. Deploy to AWS:

```bash
npm run deploy
```

## Project Structure

- `src/` - Core application source code
- `infrastructure/` - AWS CDK infrastructure definitions
- `config/` - Environment configuration files
- `scripts/` - Build and deployment scripts

## MCP Capabilities

This server provides:

- Tools for external function calls
- Resources for content access
- Protocol handling for MCP communication

## Development

- `npm run dev` - Watch mode for development
- `npm run build` - Build TypeScript code
- `npm run deploy` - Deploy infrastructure
