import { z } from "zod";

export const AgentCompletionsMessageAssistantToolCallTypeSchema = z.union([z.literal("function").describe("A function call.")]).describe("The type of tool call.").meta({ title: "agent.completions.message.AssistantToolCallType" });
export type AgentCompletionsMessageAssistantToolCallType = z.infer<typeof AgentCompletionsMessageAssistantToolCallTypeSchema>;
