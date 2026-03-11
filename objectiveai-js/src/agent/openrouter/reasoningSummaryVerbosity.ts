import { z } from "zod";

export const AgentOpenrouterReasoningSummaryVerbositySchema = z.union([z.literal("auto").describe("Let the model decide (default, normalized away)."), z.literal("concise").describe("Brief summary of reasoning."), z.literal("detailed").describe("Thorough summary of reasoning.")]).describe("Verbosity of the reasoning summary included in responses.\n\nOnly supported by some models.").meta({ title: "agent.openrouter.ReasoningSummaryVerbosity" });
export type AgentOpenrouterReasoningSummaryVerbosity = z.infer<typeof AgentOpenrouterReasoningSummaryVerbositySchema>;
