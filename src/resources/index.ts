import { fileSystemResource } from "./file-system";
import { webContentResource } from "./web-content";

export const resourceRegistry = [fileSystemResource, webContentResource];

export const getAllResources = () => resourceRegistry;
