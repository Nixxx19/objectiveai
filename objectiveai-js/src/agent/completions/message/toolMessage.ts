import { z } from "zod";
import { AgentCompletionsMessageRichContentSchema } from "./richContent";

export const AgentCompletionsMessageToolMessageSchema = z.object({
  content: AgentCompletionsMessageRichContentSchema.describe("The content of the tool response."),
  tool_call_id: z.string().describe("The ID of the tool call this message responds to."),
}).describe("A tool message containing the result of a tool call.").meta({ title: "agent.completions.message.ToolMessage" });
export type AgentCompletionsMessageToolMessage = z.infer<typeof AgentCompletionsMessageToolMessageSchema>;
