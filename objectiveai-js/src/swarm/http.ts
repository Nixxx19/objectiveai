import { ObjectiveAI, type RequestOptions } from "../client";
import type { SwarmListSwarm } from "./listSwarm";
import type { SwarmGetSwarm } from "./getSwarm";
import type { SwarmUsageSwarm } from "./usageSwarm";

export function swarmListSwarms(
  client: ObjectiveAI,
  options?: RequestOptions,
): Promise<SwarmListSwarm> {
  return client.get_unary<SwarmListSwarm>("/swarms", undefined, options);
}

export function swarmGetSwarm(
  client: ObjectiveAI,
  swarmId: string,
  options?: RequestOptions,
): Promise<SwarmGetSwarm> {
  return client.get_unary<SwarmGetSwarm>(
    `/swarms/${swarmId}`,
    undefined,
    options,
  );
}

export function swarmGetSwarmUsage(
  client: ObjectiveAI,
  swarmId: string,
  options?: RequestOptions,
): Promise<SwarmUsageSwarm> {
  return client.get_unary<SwarmUsageSwarm>(
    `/swarms/${swarmId}/usage`,
    undefined,
    options,
  );
}
