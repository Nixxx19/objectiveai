import { z } from "zod";

export const FunctionsInventionsEssayObjectSchema = z.object({
  essay: z.string(),
}).meta({ title: "functions.inventions.EssayObject" });
export type FunctionsInventionsEssayObject = z.infer<typeof FunctionsInventionsEssayObjectSchema>;
