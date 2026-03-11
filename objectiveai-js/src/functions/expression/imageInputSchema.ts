import { z } from "zod";

export const FunctionsExpressionImageInputSchemaSchema = z.object({
  description: z.string().nullable().describe("Human-readable description of the expected image.").optional(),
}).describe("Schema for an image input (URL or base64-encoded).").meta({ title: "functions.expression.ImageInputSchema" });
export type FunctionsExpressionImageInputSchema = z.infer<typeof FunctionsExpressionImageInputSchemaSchema>;
