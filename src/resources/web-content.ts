import axios from "axios";

export interface WebContentResource {
  name: string;
  description: string;
  fetchContent: (url: string) => Promise<string>;
}

export const webContentResource: WebContentResource = {
  name: "web-content",
  description: "Fetches content from web pages",
  fetchContent: async (url: string) => {
    const response = await axios.get(url);
    return response.data;
  },
};
