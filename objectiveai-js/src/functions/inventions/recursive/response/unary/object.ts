import { z } from "zod";

export const FunctionsInventionsRecursiveResponseUnaryObjectSchema = z.enum(["alpha.scalar.function.invention.recursive","alpha.vector.function.invention.recursive"]).meta({ title: "functions.inventions.recursive.response.unary.Object" });
export type FunctionsInventionsRecursiveResponseUnaryObject = z.infer<typeof FunctionsInventionsRecursiveResponseUnaryObjectSchema>;
