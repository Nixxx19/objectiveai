import { z } from "zod";

export const FunctionsInventionsResponseUnaryObjectSchema = z.enum(["alpha.scalar.function.invention","alpha.vector.function.invention"]).meta({ title: "functions.inventions.response.unary.Object" });
export type FunctionsInventionsResponseUnaryObject = z.infer<typeof FunctionsInventionsResponseUnaryObjectSchema>;
