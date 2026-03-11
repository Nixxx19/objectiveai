import { z } from "zod";

export const FunctionsInventionsEssayTasksObjectSchema = z.object({
  essay_tasks: z.string(),
}).meta({ title: "functions.inventions.EssayTasksObject" });
export type FunctionsInventionsEssayTasksObject = z.infer<typeof FunctionsInventionsEssayTasksObjectSchema>;
