import { z } from "zod";

export const FunctionsExpressionAudioInputSchemaSchema = z.object({
  description: z.string().nullable().describe("Human-readable description of the expected audio.").optional(),
}).describe("Schema for an audio input.").meta({ title: "functions.expression.AudioInputSchema" });
export type FunctionsExpressionAudioInputSchema = z.infer<typeof FunctionsExpressionAudioInputSchemaSchema>;
