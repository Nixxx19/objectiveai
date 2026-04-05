import type { TreeNode } from "./types";
import type {
  ExecutionFrame,
  ExecutionTimeline,
  AgentInfo,
  Vote,
} from "./execution";
import { aggregateScores, scoresToScalar } from "./execution";

const NANO_AGENTS: AgentInfo[] = [
  { model: "deepseek/v3.2", mode: "instruction", weight: 1.0 },
  { model: "grok/4.1-fast", mode: "json_schema", weight: 1.0 },
  { model: "openai/gpt-4o-mini", mode: "json_schema", weight: 1.0 },
  { model: "google/gemini-flash-lite", mode: "json_schema", weight: 1.0 },
];

/**
 * Generate a simulated execution timeline for any tree.
 * Walks depth-first, generates plausible votes for vector-completion nodes,
 * propagates scores up to function nodes.
 */
export function simulateTree(root: TreeNode): ExecutionTimeline {
  const frames: ExecutionFrame[] = [];
  frames.push({ nodeId: root.id, state: "pending", label: "Execution started" });

  const rootScore = simulateNode(root, frames);
  frames.push({
    nodeId: root.id,
    state: "resolved",
    output: rootScore,
    label: `Final: ${rootScore.toFixed(3)}`,
  });

  return { frames, agents: NANO_AGENTS };
}

/** Simulate a node and return its scalar output */
function simulateNode(node: TreeNode, frames: ExecutionFrame[]): number {
  if (node.kind === "vector-completion") {
    return simulateVote(node, frames);
  }

  // Function node: simulate children sequentially, average their outputs
  if (node.children.length === 0) {
    // Leaf function with no children — produce a random score
    frames.push({ nodeId: node.id, state: "pending", label: node.label });
    const score = 0.3 + seededRandom(node.id) * 0.5;
    frames.push({
      nodeId: node.id,
      state: "resolved",
      output: score,
      label: `${node.label} → ${score.toFixed(3)}`,
    });
    return score;
  }

  frames.push({ nodeId: node.id, state: "pending", label: node.label });

  const childScores: number[] = [];
  for (const child of node.children) {
    const score = simulateNode(child, frames);
    childScores.push(score);
  }

  const avg = childScores.reduce((a, b) => a + b, 0) / childScores.length;
  frames.push({
    nodeId: node.id,
    state: "resolved",
    output: avg,
    label: `${node.label} → ${avg.toFixed(3)}`,
  });

  return avg;
}

/** Simulate a vector completion vote */
function simulateVote(node: TreeNode, frames: ExecutionFrame[]): number {
  const responseCount = node.responses?.length ?? 2;
  frames.push({ nodeId: node.id, state: "voting", label: node.label });

  const allVotes: Vote[] = [];
  for (let a = 0; a < NANO_AGENTS.length; a++) {
    const agent = NANO_AGENTS[a];
    const dist = generateVoteDistribution(responseCount, node.id, a);
    const vote: Vote = {
      model: agent.model,
      vote: dist,
      weight: agent.weight,
      flatIndex: a,
      source: "live",
    };
    allVotes.push(vote);
    const agg = aggregateScores(allVotes);
    frames.push({
      nodeId: node.id,
      state: "voting",
      vote,
      scores: agg.scores,
      weights: agg.weights,
      label: `${agent.model.split("/").pop()} voted`,
    });
  }

  const finalAgg = aggregateScores(allVotes);
  const scalar = scoresToScalar(finalAgg.scores);
  frames.push({
    nodeId: node.id,
    state: "resolved",
    scores: finalAgg.scores,
    output: scalar,
    label: `${scalar.toFixed(3)}`,
  });

  return scalar;
}

/** Generate a plausible vote distribution that sums to 1 */
function generateVoteDistribution(
  count: number,
  nodeId: string,
  agentIndex: number
): number[] {
  // Use deterministic pseudo-random based on node ID and agent index
  const raw: number[] = [];
  for (let i = 0; i < count; i++) {
    raw.push(Math.max(0.01, seededRandom(`${nodeId}-${agentIndex}-${i}`)));
  }
  const sum = raw.reduce((a, b) => a + b, 0);
  return raw.map((v) => v / sum);
}

/** Simple deterministic hash-based pseudo-random [0, 1) */
function seededRandom(seed: string): number {
  let hash = 0;
  for (let i = 0; i < seed.length; i++) {
    hash = ((hash << 5) - hash + seed.charCodeAt(i)) | 0;
  }
  return ((hash & 0x7fffffff) % 10000) / 10000;
}
