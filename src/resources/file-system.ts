import * as fs from "fs";
import * as path from "path";
import { MCPResource } from "../types/mcp-sdk";

export const fileSystemResource: MCPResource = {
  name: "file-system",
  description: "Provides access to file system resources",
  readFile: async (filePath: string) => {
    return fs.readFileSync(filePath, "utf8");
  },
  listDirectory: async (dirPath: string) => {
    return fs.readdirSync(dirPath);
  },
};
