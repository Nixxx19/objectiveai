import { validateSwarm } from "../wasm/loader.js";
import type { SwarmSwarmBase } from "./swarmBase";
import type { SwarmSwarm } from "./swarm";
import type { AgentRemoteAgentWithFallbacks } from "../agent/remoteAgentWithFallbacks";

export function wasmSwarmValidateSwarm(
  swarm: SwarmSwarmBase,
  remoteAgents?: Record<string, AgentRemoteAgentWithFallbacks>,
): SwarmSwarm {
  return JSON.parse(validateSwarm(swarm, remoteAgents));
}
