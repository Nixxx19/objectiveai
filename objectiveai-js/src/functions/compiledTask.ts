import { z } from "zod";
import { FunctionsTaskSchema } from "./task";

export const FunctionsCompiledTaskSchema = z.union([FunctionsTaskSchema.describe("A single task (no mapping)."), z.array(FunctionsTaskSchema).describe("Multiple task instances from mapped execution.")]).describe("The result of compiling a task expression.\n\nTasks without a `map` field compile to a single task. Tasks with a `map`\nexpression are expanded into multiple tasks, one per integer index from\n0 to the evaluated count.").meta({ title: "functions.CompiledTask" });
export type FunctionsCompiledTask = z.infer<typeof FunctionsCompiledTaskSchema>;
