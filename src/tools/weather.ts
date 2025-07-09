import { z } from "zod";

export const weatherToolSchema = {
  location: z.string().describe("The city or location to get weather for"),
  unit: z.enum(["celsius", "fahrenheit"]).default("celsius"),
};

export const weatherToolCallback = async (args: {
  location: string;
  unit: "celsius" | "fahrenheit";
}) => {
  const { location, unit } = args;

  // Mock weather data - in real implementation, call weather API
  const mockTemp = unit === "fahrenheit" ? 72 : 22;
  const unitSymbol = unit === "fahrenheit" ? "°F" : "°C";

  return {
    content: [
      {
        type: "text" as const,
        text: `Weather in ${location}: ${mockTemp}${unitSymbol}, sunny, humidity 65%`,
      },
    ],
  };
};
