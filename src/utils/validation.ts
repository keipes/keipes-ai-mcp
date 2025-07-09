import { z } from "zod";

export const schemas = {
  mcpRequest: z.object({
    jsonrpc: z.string(),
    id: z.union([z.string(), z.number()]),
    method: z.string(),
    params: z.any().optional(),
  }),
};

export const validateRequest = (data: any, schema: z.ZodSchema) => {
  return schema.parse(data);
};
