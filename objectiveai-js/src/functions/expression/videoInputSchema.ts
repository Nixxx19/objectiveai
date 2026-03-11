import { z } from "zod";

export const FunctionsExpressionVideoInputSchemaSchema = z.object({
  description: z.string().nullable().describe("Human-readable description of the expected video.").optional(),
}).describe("Schema for a video input (URL or base64-encoded).").meta({ title: "functions.expression.VideoInputSchema" });
export type FunctionsExpressionVideoInputSchema = z.infer<typeof FunctionsExpressionVideoInputSchemaSchema>;
