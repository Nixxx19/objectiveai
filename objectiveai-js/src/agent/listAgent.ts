import { z } from "zod";
import { AgentListAgentItemSchema } from "./listAgentItem";

export const AgentListAgentSchema = z.object({
  data: z.array(AgentListAgentItemSchema).describe("The list of Agent summaries."),
}).describe("Response containing a list of Agents.").meta({ title: "agent.ListAgent" });
export type AgentListAgent = z.infer<typeof AgentListAgentSchema>;
