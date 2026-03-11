import { z } from "zod";

export const VectorCompletionsResponseStreamingObjectSchema = z.union([z.literal("vector.completion.chunk").describe("A streaming vector completion chunk.")]).describe("Object type for streaming vector completion chunks.\n\nSerializes to `\"vector.completion.chunk\"` in JSON.").meta({ title: "vector.completions.response.streaming.Object" });
export type VectorCompletionsResponseStreamingObject = z.infer<typeof VectorCompletionsResponseStreamingObjectSchema>;
