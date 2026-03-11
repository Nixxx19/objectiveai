import { z } from "zod";

export const AgentCompletionsResponseCompletionTokensDetailsSchema = z.object({
  accepted_prediction_tokens: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Tokens from accepted predictions (speculative decoding).").optional(),
  audio_tokens: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Audio output tokens.").optional(),
  reasoning_tokens: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Tokens used for reasoning/thinking.").optional(),
  rejected_prediction_tokens: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Tokens from rejected predictions (speculative decoding).").optional(),
}).describe("Detailed breakdown of completion token usage.").meta({ title: "agent.completions.response.CompletionTokensDetails" });
export type AgentCompletionsResponseCompletionTokensDetails = z.infer<typeof AgentCompletionsResponseCompletionTokensDetailsSchema>;
