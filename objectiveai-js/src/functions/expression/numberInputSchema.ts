import { z } from "zod";

export const FunctionsExpressionNumberInputSchemaSchema = z.object({
  description: z.string().nullable().describe("Human-readable description of the number.").optional(),
  minimum: z.number().meta({ format: "double" }).nullable().describe("Minimum allowed value (inclusive).").optional(),
  maximum: z.number().meta({ format: "double" }).nullable().describe("Maximum allowed value (inclusive).").optional(),
}).describe("Schema for a floating-point number input.").meta({ title: "functions.expression.NumberInputSchema" });
export type FunctionsExpressionNumberInputSchema = z.infer<typeof FunctionsExpressionNumberInputSchemaSchema>;
