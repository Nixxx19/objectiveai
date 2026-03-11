import { z } from "zod";

export const FunctionsInventionsTasksLengthObjectSchema = z.object({
  tasks_length: z.number().int().min(0).meta({ format: "uint64" }),
}).meta({ title: "functions.inventions.TasksLengthObject" });
export type FunctionsInventionsTasksLengthObject = z.infer<typeof FunctionsInventionsTasksLengthObjectSchema>;
