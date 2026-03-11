import { z } from "zod";

export const AgentUpstreamSchema = z.union([z.literal("unknown").describe("Unknown Upstream."), z.literal("openrouter").describe("OpenRouter Upstream."), z.literal("claude_agent_sdk").describe("Claude Agent SDK Upstream."), z.literal("mock").describe("Mock Upstream.")]).describe("Supported agent upstreams.").meta({ title: "agent.Upstream" });
export type AgentUpstream = z.infer<typeof AgentUpstreamSchema>;
