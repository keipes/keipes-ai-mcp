# Project Definition

This is a serverless Model Context Protocol application.

- Hosting is AWS.
- Serverless architecture.
- Uses CDK v2 for infrastructure as code.
- Golang is the primary programming language.
- Use Typescript for CDK scripts.

# Code Implementation Guidelines

When writing code, focus on minimal functional implementations:

- Write only the essential code needed for a working solution
- Avoid unnecessary error handling, logging, or defensive programming
- Skip comprehensive testing, validation, documentation, or edge case handling
- Prioritize simplicity and getting to a functional result quickly

# Phases

## Phase 1: Planning

- Create a plan.json in the root directory of this project.
- Use Plan Definition Format to outline the project structure.

### Plan Definition Format

#### Directory Definition

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
