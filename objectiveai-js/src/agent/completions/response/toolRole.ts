import { z } from "zod";

export const AgentCompletionsResponseToolRoleSchema = z.literal("tool").meta({ title: "agent.completions.response.ToolRole" });
export type AgentCompletionsResponseToolRole = z.infer<typeof AgentCompletionsResponseToolRoleSchema>;
