import { z } from "zod";
import { AgentCompletionsResponseStreamingMessageChunkSchema } from "../../../../agent/completions/response/streaming/messageChunk";
import { AgentCompletionsResponseStreamingObjectSchema } from "../../../../agent/completions/response/streaming/object";
import { AgentCompletionsResponseUsageSchema } from "../../../../agent/completions/response/usage";
import { AgentUpstreamSchema } from "../../../../agent/upstream";
import { ResponseErrorSchema } from "../../../../responseError";

export const FunctionsInventionsResponseStreamingAgentCompletionChunkSchema = z.object({
  index: z.number().int().min(0).meta({ format: "uint64" }),
  id: z.string(),
  created: z.number().int().min(0).meta({ format: "uint64" }),
  messages: z.array(AgentCompletionsResponseStreamingMessageChunkSchema),
  object: AgentCompletionsResponseStreamingObjectSchema.describe("The object type (always \"agent.completion.chunk\")."),
  usage: AgentCompletionsResponseUsageSchema.nullable().describe("Token usage (only present in the final chunk).").optional(),
  upstream: AgentUpstreamSchema.describe("Upstream provider"),
  error: ResponseErrorSchema.nullable().describe("Error details if this completion failed.").optional(),
}).describe("A chunk of a streaming agent completion response.\n\nMultiple chunks are received via Server-Sent Events and can be\naccumulated into a complete [`AgentCompletion`](response::unary::AgentCompletion)\nusing the [`push`](Self::push) method.").meta({ title: "functions.inventions.response.streaming.AgentCompletionChunk" });
export type FunctionsInventionsResponseStreamingAgentCompletionChunk = z.infer<typeof FunctionsInventionsResponseStreamingAgentCompletionChunkSchema>;
