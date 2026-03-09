import z from "zod";
import { VectorCompletionTaskSchema } from "./vectorCompletionTask";
import { FunctionExecutionTaskSchema } from "./functionExecutionTask";

export const TaskSchema = z
  .union([FunctionExecutionTaskSchema, VectorCompletionTaskSchema])
  .describe("A task execution.");
export type Task = z.infer<typeof TaskSchema>;
