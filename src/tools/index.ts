import { calculatorTool } from "./calculator";
import { weatherTool } from "./weather";

export const toolRegistry = [calculatorTool, weatherTool];

export const getAllTools = () => toolRegistry;
