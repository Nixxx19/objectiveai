import { z } from "zod";
import { FunctionsExpressionInputSchemaSchema } from "./inputSchema";

export const FunctionsExpressionArrayInputSchemaSchema = z.object({
  description: z.string().nullable().describe("Human-readable description of the array.").optional(),
  minItems: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Minimum number of items required.").optional(),
  maxItems: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Maximum number of items allowed.").optional(),
  items: z.lazy(() => FunctionsExpressionInputSchemaSchema).describe("Schema for each item in the array."),
}).describe("Schema for an array input.").meta({ title: "functions.expression.ArrayInputSchema" });
export type FunctionsExpressionArrayInputSchema = z.infer<typeof FunctionsExpressionArrayInputSchemaSchema>;
