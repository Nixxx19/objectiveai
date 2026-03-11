import { z } from "zod";

export const FunctionsExpressionIntegerInputSchemaSchema = z.object({
  description: z.string().nullable().describe("Human-readable description of the integer.").optional(),
  minimum: z.number().int().meta({ format: "int64" }).nullable().describe("Minimum allowed value (inclusive).").optional(),
  maximum: z.number().int().meta({ format: "int64" }).nullable().describe("Maximum allowed value (inclusive).").optional(),
}).describe("Schema for an integer input.").meta({ title: "functions.expression.IntegerInputSchema" });
export type FunctionsExpressionIntegerInputSchema = z.infer<typeof FunctionsExpressionIntegerInputSchemaSchema>;
