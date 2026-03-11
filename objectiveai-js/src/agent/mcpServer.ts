import { z } from "zod";

export const AgentMcpServerSchema = z.object({
  url: z.string().describe("The URL of the MCP server."),
  authorization: z.boolean().default(false).describe("Whether this MCP server uses authorization.").optional(),
}).describe("An MCP server that the agent can connect to.").meta({ title: "agent.McpServer" });
export type AgentMcpServer = z.infer<typeof AgentMcpServerSchema>;
