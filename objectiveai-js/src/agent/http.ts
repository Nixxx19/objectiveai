import { ObjectiveAI, type RequestOptions } from "../client";
import type { AgentListAgent } from "./listAgent";
import type { AgentGetAgent } from "./getAgent";
import type { AgentUsageAgent } from "./usageAgent";

export function agentListAgents(
  client: ObjectiveAI,
  options?: RequestOptions,
): Promise<AgentListAgent> {
  return client.get_unary<AgentListAgent>("/agents", undefined, options);
}

export function agentGetAgent(
  client: ObjectiveAI,
  agentId: string,
  options?: RequestOptions,
): Promise<AgentGetAgent> {
  return client.get_unary<AgentGetAgent>(`/agents/${agentId}`, undefined, options);
}

export function agentGetAgentUsage(
  client: ObjectiveAI,
  agentId: string,
  options?: RequestOptions,
): Promise<AgentUsageAgent> {
  return client.get_unary<AgentUsageAgent>(
    `/agents/${agentId}/usage`,
    undefined,
    options,
  );
}
