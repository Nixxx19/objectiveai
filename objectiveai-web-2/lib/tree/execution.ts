// ── Execution state: what happens when a function runs ──

/** A single agent's vote on a vector completion */
export interface Vote {
  model: string;
  /** Distribution over responses, sums to 1.0 */
  vote: number[];
  /** Profile weight for this agent [0, 1] */
  weight: number;
  /** Flat index in swarm */
  flatIndex: number;
  /** Vote source */
  source: "live" | "cache" | "rng";
}

/** Execution state of a single node in the tree */
export type NodeState = "idle" | "pending" | "voting" | "resolved";

/** Per-node execution data, keyed by node ID */
export interface NodeExecution {
  state: NodeState;
  /** For vector-completion nodes: accumulated votes */
  votes: Vote[];
  /** Current aggregated scores (one per response), updated after each vote */
  scores: number[];
  /** Current aggregated weights */
  weights: number[];
  /** For function nodes: resolved scalar output [0, 1] */
  output?: number;
}

/** A single frame in the execution timeline */
export interface ExecutionFrame {
  /** Which node changed */
  nodeId: string;
  /** New state for that node */
  state: NodeState;
  /** If voting: the new vote that arrived */
  vote?: Vote;
  /** Updated scores after this frame */
  scores?: number[];
  /** Updated weights after this frame */
  weights?: number[];
  /** If resolved: the final output */
  output?: number;
  /** Human-readable description of what happened */
  label: string;
}

/** Full execution timeline */
export interface ExecutionTimeline {
  frames: ExecutionFrame[];
  /** Swarm agents participating */
  agents: AgentInfo[];
}

export interface AgentInfo {
  model: string;
  mode: string;
  weight: number;
}

// ── Score aggregation (mirrors the real engine) ──

/**
 * Aggregate votes into scores using weighted averaging.
 * score[i] = sum(vote[i] * weight) / sum(weights)
 */
export function aggregateScores(votes: Vote[]): {
  scores: number[];
  weights: number[];
} {
  if (votes.length === 0) return { scores: [], weights: [] };

  const responseCount = votes[0].vote.length;
  const weights = new Array(responseCount).fill(0);
  let totalWeight = 0;

  for (const v of votes) {
    for (let i = 0; i < responseCount; i++) {
      weights[i] += v.vote[i] * v.weight;
    }
    totalWeight += v.weight;
  }

  const scores =
    totalWeight > 0
      ? weights.map((w) => w / totalWeight)
      : new Array(responseCount).fill(1 / responseCount);

  return { scores, weights };
}

/**
 * Convert a vector completion's score distribution to a scalar.
 * Default: weighted sum by position (first response = 1.0, last = 0.0).
 * This mirrors the real Starlark output expressions.
 */
export function scoresToScalar(scores: number[]): number {
  if (scores.length <= 1) return scores[0] ?? 0;
  const step = 1 / (scores.length - 1);
  let scalar = 0;
  for (let i = 0; i < scores.length; i++) {
    scalar += scores[i] * (1 - i * step);
  }
  return Math.max(0, Math.min(1, scalar));
}
