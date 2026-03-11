import { z } from "zod";
import { AgentCompletionsMessageSimpleContentSchema } from "./simpleContent";

export const AgentCompletionsMessageDeveloperMessageSchema = z.object({
  content: AgentCompletionsMessageSimpleContentSchema.describe("The message content."),
  name: z.string().nullable().describe("Optional name for the message author.").optional(),
}).describe("A developer message.").meta({ title: "agent.completions.message.DeveloperMessage" });
export type AgentCompletionsMessageDeveloperMessage = z.infer<typeof AgentCompletionsMessageDeveloperMessageSchema>;
