import { z } from "zod";
import { AgentAgentBaseSchema } from "../../agentBase";

export const AgentCompletionsRequestAgentSchema = z.union([z.string().describe("The content-addressed ID of an Agent stored in ObjectiveAI's database."), AgentAgentBaseSchema.describe("An inline Agent configuration.")]).describe("The agent to use for agent completion.\n\nCan be either:\n- An inline [`AgentBase`](super::super::super::AgentBase) configuration\n- The ID of a previously used Agent (22-character base62 string)\n\nSince IDs are content-addressed, ObjectiveAI stores Agent definitions\nwhen they are successfully used. \"Previously used\" means the ID exists in\nObjectiveAI's database from any successful use by anyone.").meta({ title: "agent.completions.request.Agent" });
export type AgentCompletionsRequestAgent = z.infer<typeof AgentCompletionsRequestAgentSchema>;
