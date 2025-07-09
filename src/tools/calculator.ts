import { z } from "zod";

export const calculatorToolSchema = {
  operation: z.enum(["add", "subtract", "multiply", "divide"]),
  a: z.number(),
  b: z.number(),
};

export const calculatorToolCallback = async (args: {
  operation: "add" | "subtract" | "multiply" | "divide";
  a: number;
  b: number;
}) => {
  const { operation, a, b } = args;

  switch (operation) {
    case "add":
      return {
        content: [{ type: "text" as const, text: `${a} + ${b} = ${a + b}` }],
      };
    case "subtract":
      return {
        content: [{ type: "text" as const, text: `${a} - ${b} = ${a - b}` }],
      };
    case "multiply":
      return {
        content: [{ type: "text" as const, text: `${a} × ${b} = ${a * b}` }],
      };
    case "divide":
      return {
        content: [
          {
            type: "text" as const,
            text:
              b !== 0 ? `${a} ÷ ${b} = ${a / b}` : "Error: Division by zero",
          },
        ],
      };
    default:
      throw new Error("Unknown operation");
  }
};
