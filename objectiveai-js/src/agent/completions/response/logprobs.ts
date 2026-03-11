import { z } from "zod";
import { AgentCompletionsResponseLogprobSchema } from "./logprob";

export const AgentCompletionsResponseLogprobsSchema = z.object({
  content: z.array(AgentCompletionsResponseLogprobSchema).nullable().describe("Log probabilities for content tokens.").optional(),
  refusal: z.array(AgentCompletionsResponseLogprobSchema).nullable().describe("Log probabilities for refusal tokens.").optional(),
}).describe("Log probabilities for generated tokens.").meta({ title: "agent.completions.response.Logprobs" });
export type AgentCompletionsResponseLogprobs = z.infer<typeof AgentCompletionsResponseLogprobsSchema>;
