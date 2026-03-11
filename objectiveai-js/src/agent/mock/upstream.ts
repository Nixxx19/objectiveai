import { z } from "zod";

export const AgentMockUpstreamSchema = z.literal("mock").describe("Mock upstream marker.").meta({ title: "agent.mock.Upstream" });
export type AgentMockUpstream = z.infer<typeof AgentMockUpstreamSchema>;
