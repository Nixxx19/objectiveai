import { z } from "zod";
import { AgentCompletionsMessageAssistantToolCallSchema } from "./assistantToolCall";
import { AgentCompletionsMessageRichContentSchema } from "./richContent";

export const AgentCompletionsMessageAssistantMessageSchema = z.object({
  content: AgentCompletionsMessageRichContentSchema.nullable().describe("The message content, if any.").optional(),
  name: z.string().nullable().describe("Optional name for the assistant.").optional(),
  refusal: z.string().nullable().describe("Refusal message if the model declined to respond.").optional(),
  tool_calls: z.array(AgentCompletionsMessageAssistantToolCallSchema).nullable().describe("Tool calls made by the assistant.").optional(),
  reasoning: z.string().nullable().describe("Reasoning content from models that support chain-of-thought.").optional(),
}).describe("An assistant message (model's previous response).").meta({ title: "agent.completions.message.AssistantMessage" });
export type AgentCompletionsMessageAssistantMessage = z.infer<typeof AgentCompletionsMessageAssistantMessageSchema>;
