import { z } from "zod";
import { AgentCompletionsResponseUsageSchema } from "../../../../agent/completions/response/usage";
import { FunctionsExecutionsResponseStreamingObjectSchema } from "./object";
import { FunctionsExecutionsResponseStreamingReasoningSummaryChunkSchema } from "./reasoningSummaryChunk";
import { FunctionsExecutionsResponseStreamingTaskChunkSchema } from "./taskChunk";
import { FunctionsExpressionTaskOutputOwnedSchema } from "../../../expression/taskOutputOwned";
import { ResponseErrorSchema } from "../../../../responseError";

export const FunctionsExecutionsResponseStreamingFunctionExecutionChunkSchema = z.object({
  id: z.string(),
  tasks: z.array(FunctionsExecutionsResponseStreamingTaskChunkSchema),
  tasks_errors: z.boolean().nullable().optional(),
  reasoning: FunctionsExecutionsResponseStreamingReasoningSummaryChunkSchema.nullable().optional(),
  output: FunctionsExpressionTaskOutputOwnedSchema.nullable().optional(),
  error: ResponseErrorSchema.nullable().optional(),
  retry_token: z.string().nullable().optional(),
  created: z.number().int().min(0).meta({ format: "uint64" }),
  function: z.string().nullable().optional(),
  profile: z.string().nullable().optional(),
  object: FunctionsExecutionsResponseStreamingObjectSchema,
  usage: AgentCompletionsResponseUsageSchema.nullable().optional(),
}).meta({ title: "functions.executions.response.streaming.FunctionExecutionChunk" });
export type FunctionsExecutionsResponseStreamingFunctionExecutionChunk = z.infer<typeof FunctionsExecutionsResponseStreamingFunctionExecutionChunkSchema>;
