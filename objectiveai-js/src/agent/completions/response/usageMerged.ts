import { merge } from "../../../merge";
import type { AgentCompletionsResponseUsage } from "./usage";
import { agentCompletionsResponseCompletionTokensDetailsMerged } from "./completionTokensDetailsMerged";
import { agentCompletionsResponsePromptTokensDetailsMerged } from "./promptTokensDetailsMerged";
import { agentCompletionsResponseCostDetailsMerged } from "./costDetailsMerged";

export function agentCompletionsResponseUsageMerged(
  a: AgentCompletionsResponseUsage,
  b: AgentCompletionsResponseUsage,
): [AgentCompletionsResponseUsage, boolean] {
  const completion_tokens = a.completion_tokens + b.completion_tokens;
  const prompt_tokens = a.prompt_tokens + b.prompt_tokens;
  const total_tokens = a.total_tokens + b.total_tokens;

  const [completion_tokens_details, c1] = merge(
    a.completion_tokens_details ?? undefined,
    b.completion_tokens_details ?? undefined,
    agentCompletionsResponseCompletionTokensDetailsMerged,
  );
  const [prompt_tokens_details, c2] = merge(
    a.prompt_tokens_details ?? undefined,
    b.prompt_tokens_details ?? undefined,
    agentCompletionsResponsePromptTokensDetailsMerged,
  );

  const cost = Number(a.cost) + Number(b.cost);

  const [cost_details, c3] = merge(
    a.cost_details ?? undefined,
    b.cost_details ?? undefined,
    agentCompletionsResponseCostDetailsMerged,
  );

  const total_cost = Number(a.total_cost) + Number(b.total_cost);

  return [{
    completion_tokens,
    prompt_tokens,
    total_tokens,
    ...(completion_tokens_details !== undefined ? { completion_tokens_details } : {}),
    ...(prompt_tokens_details !== undefined ? { prompt_tokens_details } : {}),
    cost,
    ...(cost_details !== undefined ? { cost_details } : {}),
    total_cost,
  }, true];
}
