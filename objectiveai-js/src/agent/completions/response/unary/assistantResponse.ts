import { z } from "zod";
import { AgentCompletionsMessageAssistantToolCallSchema } from "../../message/assistantToolCall";
import { AgentCompletionsMessageRichContentSchema } from "../../message/richContent";
import { AgentCompletionsResponseAssistantRoleSchema } from "../assistantRole";
import { AgentCompletionsResponseFinishReasonSchema } from "../finishReason";
import { AgentCompletionsResponseLogprobsSchema } from "../logprobs";
import { AgentCompletionsResponseUpstreamUsageSchema } from "../upstreamUsage";

export const AgentCompletionsResponseUnaryAssistantResponseSchema = z.object({
  role: AgentCompletionsResponseAssistantRoleSchema,
  index: z.number().int().min(0).meta({ format: "uint64" }),
  created: z.number().int().min(0).meta({ format: "uint64" }),
  agent: z.string(),
  model: z.string(),
  upstream_id: z.string(),
  reasoning: z.string().nullable().optional(),
  tool_calls: z.array(AgentCompletionsMessageAssistantToolCallSchema).nullable().optional(),
  content: AgentCompletionsMessageRichContentSchema.nullable().optional(),
  refusal: z.string().nullable().optional(),
  finish_reason: AgentCompletionsResponseFinishReasonSchema,
  logprobs: AgentCompletionsResponseLogprobsSchema.nullable().optional(),
  service_tier: z.string().nullable().optional(),
  system_fingerprint: z.string().nullable().optional(),
  provider: z.string().nullable().optional(),
  usage: AgentCompletionsResponseUpstreamUsageSchema.describe("Upstream usage for this assistant response (set by upstream clients)."),
}).describe("An assistant response in a unary agent completion.").meta({ title: "agent.completions.response.unary.AssistantResponse" });
export type AgentCompletionsResponseUnaryAssistantResponse = z.infer<typeof AgentCompletionsResponseUnaryAssistantResponseSchema>;
