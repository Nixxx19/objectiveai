import type { AgentCompletionsResponseCostDetails } from "./costDetails";

export function agentCompletionsResponseCostDetailsMerged(
  a: AgentCompletionsResponseCostDetails,
  b: AgentCompletionsResponseCostDetails,
): [AgentCompletionsResponseCostDetails, boolean] {
  const upstream_inference_cost = Number(a.upstream_inference_cost) + Number(b.upstream_inference_cost);
  const upstream_upstream_inference_cost = Number(a.upstream_upstream_inference_cost) + Number(b.upstream_upstream_inference_cost);
  return [{
    upstream_inference_cost,
    upstream_upstream_inference_cost,
  }, true];
}
