import { validateSwarm } from "../wasm/loader.js";
import type { SwarmSwarmBase } from "./swarmBase";
import type { SwarmSwarm } from "./swarm";

export function wasmSwarmValidateSwarm(swarm: SwarmSwarmBase): SwarmSwarm {
  return JSON.parse(validateSwarm(swarm));
}
