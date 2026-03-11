import { z } from "zod";

export const AgentOpenrouterUpstreamSchema = z.literal("openrouter").describe("OpenRouter upstream marker.").meta({ title: "agent.openrouter.Upstream" });
export type AgentOpenrouterUpstream = z.infer<typeof AgentOpenrouterUpstreamSchema>;
