import { z } from "zod";

export const FunctionsInventionsRecursiveResponseStreamingObjectSchema = z.enum(["alpha.scalar.function.invention.recursive.chunk","alpha.vector.function.invention.recursive.chunk"]).meta({ title: "functions.inventions.recursive.response.streaming.Object" });
export type FunctionsInventionsRecursiveResponseStreamingObject = z.infer<typeof FunctionsInventionsRecursiveResponseStreamingObjectSchema>;
