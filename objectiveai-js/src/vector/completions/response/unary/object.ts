import { z } from "zod";

export const VectorCompletionsResponseUnaryObjectSchema = z.union([z.literal("vector.completion").describe("A complete vector completion response.")]).describe("Object type for unary vector completion responses.\n\nSerializes to `\"vector.completion\"` in JSON.").meta({ title: "vector.completions.response.unary.Object" });
export type VectorCompletionsResponseUnaryObject = z.infer<typeof VectorCompletionsResponseUnaryObjectSchema>;
