import { z } from "zod";
import { AgentCompletionsResponseUsageSchema } from "../../../../../agent/completions/response/usage";
import { FunctionsExecutionsResponseStreamingObjectSchema } from "../../../../executions/response/streaming/object";
import { FunctionsExecutionsResponseStreamingReasoningSummaryChunkSchema } from "../../../../executions/response/streaming/reasoningSummaryChunk";
import { FunctionsExecutionsResponseStreamingTaskChunkSchema } from "../../../../executions/response/streaming/taskChunk";
import { FunctionsExpressionTaskOutputOwnedSchema } from "../../../../expression/taskOutputOwned";
import { ResponseErrorSchema } from "../../../../../responseError";

export const FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunkSchema = z.object({
  index: z.number().int().min(0).meta({ format: "uint64" }),
  dataset: z.number().int().min(0).meta({ format: "uint64" }),
  n: z.number().int().min(0).meta({ format: "uint64" }),
  retry: z.number().int().min(0).meta({ format: "uint64" }),
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
}).meta({ title: "functions.profiles.computations.response.streaming.FunctionExecutionChunk" });
export type FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunk = z.infer<typeof FunctionsProfilesComputationsResponseStreamingFunctionExecutionChunkSchema>;
