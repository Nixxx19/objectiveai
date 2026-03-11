import { z } from "zod";
import { AgentCompletionsMessageRichContentSchema } from "./richContent";

export const AgentCompletionsMessageUserMessageSchema = z.object({
  content: AgentCompletionsMessageRichContentSchema.describe("The message content (supports text, images, audio, video, files)."),
  name: z.string().nullable().describe("Optional name for the user.").optional(),
}).describe("A user message from the end user.").meta({ title: "agent.completions.message.UserMessage" });
export type AgentCompletionsMessageUserMessage = z.infer<typeof AgentCompletionsMessageUserMessageSchema>;
