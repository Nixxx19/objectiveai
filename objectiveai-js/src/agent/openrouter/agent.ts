import { z } from "zod";
import { AgentCompletionsMessageMessageSchema } from "../completions/message/message";
import { AgentMcpServerSchema } from "../mcpServer";
import { AgentOpenrouterOutputModeSchema } from "./outputMode";
import { AgentOpenrouterProviderSchema } from "./provider";
import { AgentOpenrouterReasoningSchema } from "./reasoning";
import { AgentOpenrouterStopSchema } from "./stop";
import { AgentOpenrouterUpstreamSchema } from "./upstream";
import { AgentOpenrouterVerbositySchema } from "./verbosity";

export const AgentOpenrouterAgentSchema = z.object({
  id: z.string().describe("The deterministic content-addressed ID (22-character base62 string)."),
  upstream: AgentOpenrouterUpstreamSchema.describe("The upstream provider marker."),
  model: z.string().describe("The upstream language model identifier (e.g., `\"gpt-4\"`, `\"claude-3-opus\"`)."),
  output_mode: AgentOpenrouterOutputModeSchema.describe("The output mode for vector completions. Ignored for agent completions.").optional(),
  synthetic_reasoning: z.boolean().nullable().describe("Enable synthetic reasoning for non-reasoning LLMs.\n\n**Vector completions only.** Ignored for agent completions.").optional(),
  top_logprobs: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Number of top log probabilities to return (2-20).\n\n**Vector completions only.** Ignored for agent completions.").optional(),
  prefix_messages: z.array(AgentCompletionsMessageMessageSchema).nullable().describe("Messages prepended to the user's prompt.").optional(),
  post_system_prefix_messages: z.array(AgentCompletionsMessageMessageSchema).nullable().describe("Messages inserted after the leading chain of system/developer messages.").optional(),
  suffix_messages: z.array(AgentCompletionsMessageMessageSchema).nullable().describe("Messages appended after the user's prompt.").optional(),
  mcp_servers: z.array(AgentMcpServerSchema).nullable().describe("MCP servers the agent can connect to.").optional(),
  frequency_penalty: z.number().meta({ format: "double" }).nullable().describe("Penalizes tokens based on their frequency in the output so far (-2.0 to 2.0).").optional(),
  logit_bias: z.record(z.string(), z.number().int().meta({ format: "int64" })).nullable().describe("Token ID to bias mapping (-100 to 100). Positive values increase likelihood.").optional(),
  max_completion_tokens: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Maximum tokens in the completion.").optional(),
  presence_penalty: z.number().meta({ format: "double" }).nullable().describe("Penalizes tokens based on their presence in the output so far (-2.0 to 2.0).").optional(),
  stop: AgentOpenrouterStopSchema.nullable().describe("Stop sequences that halt generation.").optional(),
  temperature: z.number().meta({ format: "double" }).nullable().describe("Sampling temperature (0.0 to 2.0). Higher = more random.").optional(),
  top_p: z.number().meta({ format: "double" }).nullable().describe("Nucleus sampling probability (0.0 to 1.0).").optional(),
  max_tokens: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Maximum tokens (OpenRouter variant of max_completion_tokens).").optional(),
  min_p: z.number().meta({ format: "double" }).nullable().describe("Minimum probability threshold for sampling (0.0 to 1.0).").optional(),
  provider: AgentOpenrouterProviderSchema.nullable().describe("Provider routing preferences.").optional(),
  reasoning: AgentOpenrouterReasoningSchema.nullable().describe("Reasoning/thinking configuration for supported models.").optional(),
  repetition_penalty: z.number().meta({ format: "double" }).nullable().describe("Repetition penalty (0.0 to 2.0). Values > 1.0 penalize repetition.").optional(),
  top_a: z.number().meta({ format: "double" }).nullable().describe("Top-a sampling parameter (0.0 to 1.0).").optional(),
  top_k: z.number().int().min(0).meta({ format: "uint64" }).nullable().describe("Top-k sampling: only consider the k most likely tokens.").optional(),
  verbosity: AgentOpenrouterVerbositySchema.nullable().describe("Output verbosity hint for supported models.").optional(),
}).describe("A validated OpenRouter Agent with its computed content-addressed ID.").meta({ title: "agent.openrouter.Agent" });
export type AgentOpenrouterAgent = z.infer<typeof AgentOpenrouterAgentSchema>;
