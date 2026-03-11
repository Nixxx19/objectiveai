import { z } from "zod";

export const FunctionsFunctionTypeSchema = z.enum(["scalar.function","vector.function"]).meta({ title: "functions.FunctionType" });
export type FunctionsFunctionType = z.infer<typeof FunctionsFunctionTypeSchema>;
