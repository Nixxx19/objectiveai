import { z } from "zod";

export const AgentCompletionsMessageAssistantToolCallFunctionDeltaSchema = z.object({
  name: z.string().nullable().describe("The function name (only present in the first delta).").optional(),
  arguments: z.string().nullable().describe("The arguments being streamed (accumulated across deltas).").optional(),
}).describe("Function call details in a streaming tool call.").meta({ title: "agent.completions.message.AssistantToolCallFunctionDelta" });
export type AgentCompletionsMessageAssistantToolCallFunctionDelta = z.infer<typeof AgentCompletionsMessageAssistantToolCallFunctionDeltaSchema>;
