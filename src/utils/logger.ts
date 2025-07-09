import { Logger } from "@aws-lambda-powertools/logger";

export const logger = new Logger({
  serviceName: "keipes-ai-mcp",
  logLevel: (process.env.LOG_LEVEL as any) || "INFO",
});
