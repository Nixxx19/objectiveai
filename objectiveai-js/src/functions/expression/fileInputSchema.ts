import { z } from "zod";

export const FunctionsExpressionFileInputSchemaSchema = z.object({
  description: z.string().nullable().describe("Human-readable description of the expected file.").optional(),
}).describe("Schema for a file input.").meta({ title: "functions.expression.FileInputSchema" });
export type FunctionsExpressionFileInputSchema = z.infer<typeof FunctionsExpressionFileInputSchemaSchema>;
