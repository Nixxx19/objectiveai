import { z } from "zod";
import { AgentCompletionsResponseCompletionTokensDetailsSchema } from "./completionTokensDetails";
import { AgentCompletionsResponseCostDetailsSchema } from "./costDetails";
import { AgentCompletionsResponsePromptTokensDetailsSchema } from "./promptTokensDetails";

export const AgentCompletionsResponseUsageSchema = z.object({
  completion_tokens: z.number().int().min(0).meta({ format: "uint64" }).describe("Total tokens generated across all assistant responses."),
  prompt_tokens: z.number().int().min(0).meta({ format: "uint64" }).describe("Total prompt tokens across all assistant responses."),
  total_tokens: z.number().int().min(0).meta({ format: "uint64" }).describe("Sum of completion and prompt tokens."),
  completion_tokens_details: AgentCompletionsResponseCompletionTokensDetailsSchema.nullable().describe("Breakdown of completion tokens (reasoning, audio, etc.) if available.").optional(),
  prompt_tokens_details: AgentCompletionsResponsePromptTokensDetailsSchema.nullable().describe("Breakdown of prompt tokens (cached, audio, etc.) if available.").optional(),
  cost: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("Cost charged by ObjectiveAI for this request."),
  cost_details: AgentCompletionsResponseCostDetailsSchema.nullable().describe("Breakdown of upstream and upstream_upstream costs if available.").optional(),
  total_cost: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("Total cost including upstream provider charges. Only differs from `cost`\nwhen using BYOK (Bring Your Own Key)."),
}).describe("Aggregated token and cost usage for an agent completion.\n\nThis is the \"primary\" usage type that aggregates across all upstream\nassistant responses within a single agent completion.").meta({ title: "agent.completions.response.Usage" });
export type AgentCompletionsResponseUsage = z.infer<typeof AgentCompletionsResponseUsageSchema>;
