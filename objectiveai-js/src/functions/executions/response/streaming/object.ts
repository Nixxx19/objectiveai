import { z } from "zod";

export const FunctionsExecutionsResponseStreamingObjectSchema = z.enum(["scalar.function.execution.chunk","vector.function.execution.chunk"]).meta({ title: "functions.executions.response.streaming.Object" });
export type FunctionsExecutionsResponseStreamingObject = z.infer<typeof FunctionsExecutionsResponseStreamingObjectSchema>;
