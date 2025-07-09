import * as fs from "fs";
import * as path from "path";

export interface FileSystemResource {
  name: string;
  description: string;
  readFile: (filePath: string) => Promise<string>;
  listDirectory: (dirPath: string) => Promise<string[]>;
}

export const fileSystemResource: FileSystemResource = {
  name: "file-system",
  description: "Provides access to file system resources",
  readFile: async (filePath: string) => {
    return fs.readFileSync(filePath, "utf8");
  },
  listDirectory: async (dirPath: string) => {
    return fs.readdirSync(dirPath);
  },
};
