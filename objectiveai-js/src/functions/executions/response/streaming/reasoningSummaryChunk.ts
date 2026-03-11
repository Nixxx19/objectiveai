import { z } from "zod";
import { AgentCompletionsResponseStreamingMessageChunkSchema } from "../../../../agent/completions/response/streaming/messageChunk";
import { AgentCompletionsResponseStreamingObjectSchema } from "../../../../agent/completions/response/streaming/object";
import { AgentCompletionsResponseUsageSchema } from "../../../../agent/completions/response/usage";
import { AgentUpstreamSchema } from "../../../../agent/upstream";
import { ResponseErrorSchema } from "../../../../responseError";

export const FunctionsExecutionsResponseStreamingReasoningSummaryChunkSchema = z.object({
  id: z.string(),
  created: z.number().int().min(0).meta({ format: "uint64" }),
  messages: z.array(AgentCompletionsResponseStreamingMessageChunkSchema),
  object: AgentCompletionsResponseStreamingObjectSchema.describe("The object type (always \"agent.completion.chunk\")."),
  usage: AgentCompletionsResponseUsageSchema.nullable().describe("Token usage (only present in the final chunk).").optional(),
  upstream: AgentUpstreamSchema.describe("Upstream provider"),
  error: ResponseErrorSchema.nullable().optional(),
}).describe("A chunk of a streaming agent completion response.\n\nMultiple chunks are received via Server-Sent Events and can be\naccumulated into a complete [`AgentCompletion`](response::unary::AgentCompletion)\nusing the [`push`](Self::push) method.").meta({ title: "functions.executions.response.streaming.ReasoningSummaryChunk" });
export type FunctionsExecutionsResponseStreamingReasoningSummaryChunk = z.infer<typeof FunctionsExecutionsResponseStreamingReasoningSummaryChunkSchema>;
