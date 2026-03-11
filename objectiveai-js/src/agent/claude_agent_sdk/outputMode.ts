import { z } from "zod";

export const AgentClaudeAgentSdkOutputModeSchema = z.union([z.literal("instruction").describe("The model is instructed via the prompt to output a specific key.\n\nThis is the default and most widely supported mode.")]).describe("The method used to constrain LLM output to valid response keys.\n\nIn vector completions, the model must select from a predefined set of\nresponses. This enum controls *how* that constraint is enforced.\n\n**Note:** This setting is only relevant for vector completions and is\ncompletely ignored for agent completions.").meta({ title: "agent.claude_agent_sdk.OutputMode" });
export type AgentClaudeAgentSdkOutputMode = z.infer<typeof AgentClaudeAgentSdkOutputModeSchema>;
