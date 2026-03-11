import { z } from "zod";

export const AgentCompletionsResponseUnaryObjectSchema = z.union([z.literal("agent.completion").describe("A agent completion object.")]).describe("The object type for agent completion responses.").meta({ title: "agent.completions.response.unary.Object" });
export type AgentCompletionsResponseUnaryObject = z.infer<typeof AgentCompletionsResponseUnaryObjectSchema>;
