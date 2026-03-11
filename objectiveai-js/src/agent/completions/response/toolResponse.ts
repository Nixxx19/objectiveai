import { z } from "zod";
import { AgentCompletionsMessageRichContentSchema } from "../message/richContent";
import { AgentCompletionsResponseToolRoleSchema } from "./toolRole";

export const AgentCompletionsResponseToolResponseSchema = z.object({
  role: AgentCompletionsResponseToolRoleSchema,
  index: z.number().int().min(0).meta({ format: "uint64" }),
  content: AgentCompletionsMessageRichContentSchema.describe("The content of the tool response."),
  tool_call_id: z.string().describe("The ID of the tool call this message responds to."),
}).describe("A tool message containing the result of a tool call.").meta({ title: "agent.completions.response.ToolResponse" });
export type AgentCompletionsResponseToolResponse = z.infer<typeof AgentCompletionsResponseToolResponseSchema>;
