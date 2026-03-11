import { z } from "zod";

export const FunctionsInventionsResponseStreamingObjectSchema = z.enum(["alpha.scalar.function.invention.chunk","alpha.vector.function.invention.chunk"]).meta({ title: "functions.inventions.response.streaming.Object" });
export type FunctionsInventionsResponseStreamingObject = z.infer<typeof FunctionsInventionsResponseStreamingObjectSchema>;
