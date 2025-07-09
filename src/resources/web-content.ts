import axios from "axios";
import { MCPResource } from "../types/mcp-sdk";

export const webContentResource: MCPResource = {
  name: "web-content",
  description: "Fetches content from web pages",
  fetchContent: async (url: string) => {
    const response = await axios.get(url);
    return response.data;
  },
};
