import { z } from "zod";

export const FunctionsUsageFunctionSchema = z.object({
  requests: z.number().int().min(0).meta({ format: "uint64" }).describe("Total number of requests made with this function."),
  completion_tokens: z.number().int().min(0).meta({ format: "uint64" }).describe("Total completion tokens used."),
  prompt_tokens: z.number().int().min(0).meta({ format: "uint64" }).describe("Total prompt tokens used."),
  total_cost: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("Total cost incurred."),
}).describe("Usage statistics for a function.").meta({ title: "functions.UsageFunction" });
export type FunctionsUsageFunction = z.infer<typeof FunctionsUsageFunctionSchema>;
