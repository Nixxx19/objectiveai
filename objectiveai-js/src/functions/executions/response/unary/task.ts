import { z } from "zod";
import { FunctionsExecutionsResponseUnaryFunctionExecutionTaskSchema } from "./functionExecutionTask";
import { FunctionsExecutionsResponseUnaryVectorCompletionTaskSchema } from "./vectorCompletionTask";

export const FunctionsExecutionsResponseUnaryTaskSchema = z.union([FunctionsExecutionsResponseUnaryFunctionExecutionTaskSchema, FunctionsExecutionsResponseUnaryVectorCompletionTaskSchema]).meta({ title: "functions.executions.response.unary.Task" });
export type FunctionsExecutionsResponseUnaryTask = z.infer<typeof FunctionsExecutionsResponseUnaryTaskSchema>;
