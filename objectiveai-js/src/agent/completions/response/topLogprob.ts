import { z } from "zod";

export const AgentCompletionsResponseTopLogprobSchema = z.object({
  token: z.string().describe("The token string."),
  bytes: z.array(z.number().int().min(0).max(255).meta({ format: "uint8" })).nullable().describe("The raw bytes of the token.").optional(),
  logprob: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).nullable().describe("The log probability of this token.").optional(),
}).describe("A top alternative token with its log probability.").meta({ title: "agent.completions.response.TopLogprob" });
export type AgentCompletionsResponseTopLogprob = z.infer<typeof AgentCompletionsResponseTopLogprobSchema>;
