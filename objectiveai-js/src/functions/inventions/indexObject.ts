import { z } from "zod";

export const FunctionsInventionsIndexObjectSchema = z.object({
  index: z.number().int().min(0).meta({ format: "uint64" }),
}).meta({ title: "functions.inventions.IndexObject" });
export type FunctionsInventionsIndexObject = z.infer<typeof FunctionsInventionsIndexObjectSchema>;
