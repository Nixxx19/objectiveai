import { z } from "zod";
import { AgentCompletionsResponseUsageSchema } from "../../../../agent/completions/response/usage";
import { FunctionsExecutionsResponseUnaryObjectSchema } from "./object";
import { FunctionsExecutionsResponseUnaryReasoningSummarySchema } from "./reasoningSummary";
import { FunctionsExecutionsResponseUnaryTaskSchema } from "./task";
import { FunctionsExpressionTaskOutputOwnedSchema } from "../../../expression/taskOutputOwned";
import { ResponseErrorSchema } from "../../../../responseError";

export const FunctionsExecutionsResponseUnaryFunctionExecutionTaskSchema = z.object({
  index: z.number().int().min(0).meta({ format: "uint64" }),
  task_index: z.number().int().min(0).meta({ format: "uint64" }),
  task_path: z.array(z.number().int().min(0).meta({ format: "uint64" })),
  swiss_pool_index: z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  swiss_round: z.number().int().min(0).meta({ format: "uint64" }).nullable().optional(),
  id: z.string().describe("Unique identifier for this execution."),
  tasks: z.array(z.lazy(() => FunctionsExecutionsResponseUnaryTaskSchema)).describe("Results from each task in the function."),
  tasks_errors: z.boolean().describe("Whether any tasks encountered errors."),
  reasoning: FunctionsExecutionsResponseUnaryReasoningSummarySchema.nullable().describe("Reasoning summary if reasoning was enabled.").optional(),
  output: FunctionsExpressionTaskOutputOwnedSchema.describe("The final output (scalar or vector score)."),
  error: ResponseErrorSchema.nullable().describe("Error details if the execution failed.").optional(),
  retry_token: z.string().nullable().describe("Token for retrying this execution with cached votes.").optional(),
  created: z.number().int().min(0).meta({ format: "uint64" }).describe("Unix timestamp when the execution was created."),
  function: z.string().nullable().describe("ID of the function used (if remote).").optional(),
  profile: z.string().nullable().describe("ID of the profile used (if remote).").optional(),
  object: FunctionsExecutionsResponseUnaryObjectSchema.describe("Object type identifier."),
  usage: AgentCompletionsResponseUsageSchema.describe("Aggregated token and cost usage."),
}).describe("A complete function execution response (non-streaming).").meta({ title: "functions.executions.response.unary.FunctionExecutionTask" });
export type FunctionsExecutionsResponseUnaryFunctionExecutionTask = z.infer<typeof FunctionsExecutionsResponseUnaryFunctionExecutionTaskSchema>;
