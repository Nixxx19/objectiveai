import { z } from "zod";

export const FunctionsExecutionsResponseUnaryObjectSchema = z.enum(["scalar.function.execution","vector.function.execution"]).meta({ title: "functions.executions.response.unary.Object" });
export type FunctionsExecutionsResponseUnaryObject = z.infer<typeof FunctionsExecutionsResponseUnaryObjectSchema>;
