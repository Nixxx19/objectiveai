import { z } from "zod";

export const AgentListAgentItemSchema = z.object({
  id: z.string().describe("The unique content-addressed ID of the Agent."),
}).describe("Summary information for a listed Agent.").meta({ title: "agent.ListAgentItem" });
export type AgentListAgentItem = z.infer<typeof AgentListAgentItemSchema>;
