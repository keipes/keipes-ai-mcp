# Copilot Instructions

- Use extreme brevity at all times.

# Project Definition

This is a Model Context Protocol server.

- Java is the primary programming language.
- Prefer Asynchronous programming.
- Use Netty.
- User Reactor Netty
- Use the official MCP Java SDK - `io.modelcontextprotocol.sdk:mcp:0.8.0`

# Code Implementation Guidelines

When writing code, focus on minimal functional implementations:

- Write only the essential code needed for a working solution.
- Avoid unnecessary error handling, logging, or defensive programming.
- Skip comprehensive testing, validation, documentation, or edge case handling.
- Prioritize simplicity.
- Prioritize separation of concerns.
- Reduce amount of code to the bare minimum required for functionality.

# Resources

MCP Documentation

- https://modelcontextprotocol.io/specification/2025-06-18
- https://modelcontextprotocol.io/sdk/java/mcp-overview
- https://modelcontextprotocol.io/sdk/java/mcp-server

Reactor Documentation

- https://projectreactor.io/docs/core/release/reference/reactiveProgramming.html

Reactor Netty Documentation

- https://projectreactor.io/docs/netty/release/reference/http-server.html

# Phases

## Phase 1: Planning

- Review Resources.
- Understand the project requirements and architecture.
- Familiarize yourself with the Model Context Protocol and its components.
- Determine the structure of the project, including modules and their relationships.
- Define the roles and responsibilities of each module in the project.
- Establish the communication protocols between modules.
- Identify the data flow and how data will be processed within the application.
- If you need additional context or information, ask for it before proceeding with the implementation.

- Create a plan.json in the root directory of this project.
- Use Plan Definition Format to outline the project structure.

### Plan Definition Format

```
{
    "directory": "<module path>",
    "description"?: "<Brief description of the module>",
    "files"?: [
        "<file1>",
        "<file2>"
    ],
    "subDirectories"?: [
        <directory1>,
        <directory2>
    ]
}
```

#### File Definition

File: A specific file within a module that contains code or configuration.

```
{
    "name": "<file name>",
    "description"?: "<Brief description of the file>",
    "dependencies"?: ["<dependency1>", "<dependency2>"],
    "exports"?: ["<export1>", "<export2>"],
}
```

## Phase 2: Implementation

- Implement the project structure as defined in the plan.json.
