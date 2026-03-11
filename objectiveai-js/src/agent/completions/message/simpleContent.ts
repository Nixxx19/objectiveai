import { z } from "zod";
import { AgentCompletionsMessageSimpleContentPartSchema } from "./simpleContentPart";

export const AgentCompletionsMessageSimpleContentSchema = z.union([z.string().describe("Plain text content."), z.array(AgentCompletionsMessageSimpleContentPartSchema).describe("Multi-part text content.")]).describe("Simple text content for system/developer messages.").meta({ title: "agent.completions.message.SimpleContent" });
export type AgentCompletionsMessageSimpleContent = z.infer<typeof AgentCompletionsMessageSimpleContentSchema>;
