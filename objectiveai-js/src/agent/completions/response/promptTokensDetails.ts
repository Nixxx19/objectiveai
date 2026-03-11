import { z } from "zod";

export const AgentCompletionsResponsePromptTokensDetailsSchema = z.object({
  audio_tokens: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Audio input tokens.").optional(),
  cached_tokens: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Tokens served from cache.").optional(),
  cache_write_tokens: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Tokens written to cache.").optional(),
  video_tokens: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Video input tokens.").optional(),
}).describe("Detailed breakdown of prompt token usage.").meta({ title: "agent.completions.response.PromptTokensDetails" });
export type AgentCompletionsResponsePromptTokensDetails = z.infer<typeof AgentCompletionsResponsePromptTokensDetailsSchema>;
