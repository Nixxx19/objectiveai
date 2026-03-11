import { z } from "zod";
import { AgentCompletionsResponseUsageSchema } from "../../../../../agent/completions/response/usage";
import { FunctionsFullRemoteFunctionSchema } from "../../../../fullRemoteFunction";
import { FunctionsInventionsResponseStreamingAgentCompletionChunkSchema } from "../../../response/streaming/agentCompletionChunk";
import { FunctionsInventionsResponseStreamingObjectSchema } from "../../../response/streaming/object";
import { FunctionsInventionsStateStateSchema } from "../../../state/state";
import { FunctionsRemoteFunctionPathSchema } from "../../../../remoteFunctionPath";
import { ResponseErrorSchema } from "../../../../../responseError";

export const FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunkSchema = z.object({
  index: z.number().int().min(0).meta({ format: "uint64" }),
  id: z.string(),
  completions: z.array(FunctionsInventionsResponseStreamingAgentCompletionChunkSchema),
  state: FunctionsInventionsStateStateSchema.nullable().optional(),
  path: FunctionsRemoteFunctionPathSchema.nullable().optional(),
  function: FunctionsFullRemoteFunctionSchema.nullable().optional(),
  created: z.number().int().min(0).meta({ format: "uint64" }),
  object: FunctionsInventionsResponseStreamingObjectSchema,
  usage: AgentCompletionsResponseUsageSchema.nullable().optional(),
  error: ResponseErrorSchema.nullable().optional(),
}).meta({ title: "functions.inventions.recursive.response.streaming.FunctionInventionChunk" });
export type FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunk = z.infer<typeof FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunkSchema>;
