# Project Definition

This is a serverless Model Context Protocol application.

- Hosting is AWS.
- Serverless architecture.
- Uses CDK v2 for infrastructure as code.
- Typescript is the primary programming language.

# Code Implementation Guidelines

When writing code, focus on minimal functional implementations:

- Write only the essential code needed for a working solution.
- Avoid unnecessary error handling, logging, or defensive programming.
- Skip comprehensive testing, validation, documentation, or edge case handling.
- Prioritize simplicity.
- Reduce amount of code to the bare minimum required for functionality.

# Phases

## Phase 1: Planning

- Understand the project requirements and architecture.
- Familiarize yourself with the Model Context Protocol and its components.
- Review the AWS services that will be used in the project.
- Ensure you have the necessary permissions and access to AWS resources.
- Identify the key components of the serverless architecture.
- Determine the structure of the project, including modules and their relationships.
- Define the roles and responsibilities of each module in the project.
- Establish the communication protocols between modules.
- Identify the data flow and how data will be processed within the application.
- Review the AWS CDK v2 documentation for Typescript to understand how to define infrastructure as code.
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
