import { z } from "zod";

export const AgentCompletionsRequestResponseFormatSchema = z.union([z.object({
  type: z.literal("text"),
}).describe("Plain text response (default)."), z.object({
  type: z.literal("json_object"),
}).describe("Response must be valid JSON."), z.object({
  schema: z.record(z.string(), z.unknown()).describe("The JSON Schema definition."),
  type: z.literal("json_schema"),
}).describe("Response must conform to a JSON schema."), z.object({
  grammar: z.string(),
  type: z.literal("grammar"),
}).describe("Response must conform to a grammar."), z.object({
  type: z.literal("python"),
}).describe("Response must be valid Python code."), z.object({
  name: z.string().describe("The name of the tool."),
  description: z.string().describe("A description of the tool."),
  schema: z.record(z.string(), z.unknown()).describe("The JSON Schema definition."),
  required: z.boolean().nullable().describe("Whether the tool MUST be called.").optional(),
  type: z.literal("tool_call"),
}).describe("The final assistant message will contain this tool call")]).describe("The format of the model's response.").meta({ title: "agent.completions.request.ResponseFormat" });
export type AgentCompletionsRequestResponseFormat = z.infer<typeof AgentCompletionsRequestResponseFormatSchema>;
