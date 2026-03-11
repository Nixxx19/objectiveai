import { z } from "zod";

export const AgentCompletionsResponseAssistantRoleSchema = z.union([z.literal("assistant").describe("The assistant role.")]).describe("The role of a message in a response (always \"assistant\").").meta({ title: "agent.completions.response.AssistantRole" });
export type AgentCompletionsResponseAssistantRole = z.infer<typeof AgentCompletionsResponseAssistantRoleSchema>;
