import { z } from "zod";
import { AgentCompletionsMessageMessageSchema } from "../message/message";
import { AgentCompletionsRequestAgentSchema } from "./agent";
import { AgentCompletionsRequestProviderSchema } from "./provider";
import { AgentCompletionsRequestResponseFormatParamSchema } from "./responseFormatParam";

export const AgentCompletionsRequestAgentCompletionCreateParamsSchema = z.object({
  messages: z.array(AgentCompletionsMessageMessageSchema).describe("The conversation messages."),
  provider: AgentCompletionsRequestProviderSchema.nullable().describe("Provider routing preferences.").optional(),
  agent: AgentCompletionsRequestAgentSchema.describe("The agent to use (inline Agent or stored ID)."),
  agents: z.array(AgentCompletionsRequestAgentSchema).nullable().describe("Alternative agents to try if the primary agent fails.").optional(),
  response_format: AgentCompletionsRequestResponseFormatParamSchema.nullable().describe("Output format constraints (text, JSON, or JSON schema).").optional(),
  seed: z.number().int().meta({ format: "int64" }).nullable().describe("Random seed for deterministic generation.").optional(),
  stream: z.boolean().nullable().describe("Whether to stream the response.").optional(),
  mcp_server_authorization: z.record(z.string(), z.string()).nullable().describe("Map from MCP server URL to authorization header value.").optional(),
}).describe("Parameters for creating a agent completion.").meta({ title: "agent.completions.request.AgentCompletionCreateParams" });
export type AgentCompletionsRequestAgentCompletionCreateParams = z.infer<typeof AgentCompletionsRequestAgentCompletionCreateParamsSchema>;
