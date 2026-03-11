import { z } from "zod";

export const FunctionsExpressionBooleanInputSchemaSchema = z.object({
  description: z.string().nullable().describe("Human-readable description of the boolean.").optional(),
}).describe("Schema for a boolean input.").meta({ title: "functions.expression.BooleanInputSchema" });
export type FunctionsExpressionBooleanInputSchema = z.infer<typeof FunctionsExpressionBooleanInputSchemaSchema>;
