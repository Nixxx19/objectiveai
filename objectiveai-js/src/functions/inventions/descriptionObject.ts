import { z } from "zod";

export const FunctionsInventionsDescriptionObjectSchema = z.object({
  description: z.string(),
}).meta({ title: "functions.inventions.DescriptionObject" });
export type FunctionsInventionsDescriptionObject = z.infer<typeof FunctionsInventionsDescriptionObjectSchema>;
