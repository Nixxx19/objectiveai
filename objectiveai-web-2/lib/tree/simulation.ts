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

// Path-based IDs matching buildTree output:
const ROOT = "root";
const SPAM = "root.0";
const SPAM_BC = "root.0.0";
const SPAM_BC_VC = "root.0.0.0";
const SPAM_EI = "root.0.1";
const SPAM_EI_VC = "root.0.1.0";
const TRIPLE = "root.1";
const TRIPLE_BC = "root.1.0";
const TRIPLE_BC_VC = "root.1.0.0";
const TRIPLE_FS = "root.1.1";
const TRIPLE_FS_VC = "root.1.1.0";
const TRIPLE_SC = "root.1.2";
const TRIPLE_SC_VC = "root.1.2.0";

export function buildSimulation(): ExecutionTimeline {
  const frames: ExecutionFrame[] = [];

  // ── Phase 1: spam-importance-branch ──

  frames.push({ nodeId: ROOT, state: "pending", label: "Execution started" });
  frames.push({ nodeId: SPAM, state: "pending", label: "spam-importance-branch" });
  frames.push({ nodeId: SPAM_BC, state: "pending", label: "binary-classifier" });
  frames.push({ nodeId: SPAM_BC_VC, state: "voting", label: "Yes or No?" });

  const bcVotes1: Vote[] = [
    { model: "deepseek/v3.2", vote: [0.85, 0.15], weight: 1.0, flatIndex: 0, source: "live" },
    { model: "grok/4.1-fast", vote: [1.0, 0.0], weight: 1.0, flatIndex: 1, source: "live" },
    { model: "openai/gpt-4o-mini", vote: [0.72, 0.28], weight: 1.0, flatIndex: 2, source: "live" },
    { model: "google/gemini-flash-lite", vote: [0.91, 0.09], weight: 1.0, flatIndex: 3, source: "live" },
  ];
  addVoteFrames(frames, SPAM_BC_VC, bcVotes1);

  const bc1Agg = aggregateScores(bcVotes1);
  const bc1Scalar = scoresToScalar(bc1Agg.scores);
  frames.push({ nodeId: SPAM_BC_VC, state: "resolved", scores: bc1Agg.scores, output: bc1Scalar, label: fmt(bc1Scalar) });
  frames.push({ nodeId: SPAM_BC, state: "resolved", output: bc1Scalar, label: `binary-classifier → ${fmt(bc1Scalar)}` });

  frames.push({ nodeId: SPAM_EI, state: "pending", label: "email-importance" });
  frames.push({ nodeId: SPAM_EI_VC, state: "voting", label: "Critical / Important / Normal / Low / Ignore" });

  const eiVotes: Vote[] = [
    { model: "deepseek/v3.2", vote: [0.05, 0.10, 0.15, 0.30, 0.40], weight: 1.0, flatIndex: 0, source: "live" },
    { model: "grok/4.1-fast", vote: [0.02, 0.08, 0.20, 0.25, 0.45], weight: 1.0, flatIndex: 1, source: "live" },
    { model: "openai/gpt-4o-mini", vote: [0.01, 0.05, 0.14, 0.35, 0.45], weight: 1.0, flatIndex: 2, source: "live" },
    { model: "google/gemini-flash-lite", vote: [0.0, 0.12, 0.18, 0.28, 0.42], weight: 1.0, flatIndex: 3, source: "live" },
  ];
  addVoteFrames(frames, SPAM_EI_VC, eiVotes);

  const eiAgg = aggregateScores(eiVotes);
  const eiScalar = scoresToScalar(eiAgg.scores);
  frames.push({ nodeId: SPAM_EI_VC, state: "resolved", scores: eiAgg.scores, output: eiScalar, label: fmt(eiScalar) });
  frames.push({ nodeId: SPAM_EI, state: "resolved", output: eiScalar, label: `email-importance → ${fmt(eiScalar)}` });

  const spamScore = (bc1Scalar + eiScalar) / 2;
  frames.push({ nodeId: SPAM, state: "resolved", output: spamScore, label: `spam-importance → ${fmt(spamScore)}` });

  // ── Phase 2: triple-classifier-branch ──

  frames.push({ nodeId: TRIPLE, state: "pending", label: "triple-classifier-branch" });
  frames.push({ nodeId: TRIPLE_BC, state: "pending", label: "binary-classifier" });
  frames.push({ nodeId: TRIPLE_BC_VC, state: "voting", label: "Yes or No?" });

  const bcVotes2: Vote[] = [
    { model: "deepseek/v3.2", vote: [0.80, 0.20], weight: 1.0, flatIndex: 0, source: "live" },
    { model: "grok/4.1-fast", vote: [0.95, 0.05], weight: 1.0, flatIndex: 1, source: "live" },
    { model: "openai/gpt-4o-mini", vote: [0.68, 0.32], weight: 1.0, flatIndex: 2, source: "live" },
    { model: "google/gemini-flash-lite", vote: [0.88, 0.12], weight: 1.0, flatIndex: 3, source: "live" },
  ];
  addVoteFrames(frames, TRIPLE_BC_VC, bcVotes2);

  const bc2Agg = aggregateScores(bcVotes2);
  const bc2Scalar = scoresToScalar(bc2Agg.scores);
  frames.push({ nodeId: TRIPLE_BC_VC, state: "resolved", scores: bc2Agg.scores, output: bc2Scalar, label: fmt(bc2Scalar) });
  frames.push({ nodeId: TRIPLE_BC, state: "resolved", output: bc2Scalar, label: `binary-classifier → ${fmt(bc2Scalar)}` });

  frames.push({ nodeId: TRIPLE_FS, state: "pending", label: "five-star-rating" });
  frames.push({ nodeId: TRIPLE_FS_VC, state: "voting", label: "★★★★★ / ★★★★ / ★★★ / ★★ / ★" });

  const fsVotes: Vote[] = [
    { model: "deepseek/v3.2", vote: [0.05, 0.15, 0.40, 0.30, 0.10], weight: 1.0, flatIndex: 0, source: "live" },
    { model: "grok/4.1-fast", vote: [0.02, 0.10, 0.35, 0.38, 0.15], weight: 1.0, flatIndex: 1, source: "live" },
    { model: "openai/gpt-4o-mini", vote: [0.08, 0.20, 0.42, 0.22, 0.08], weight: 1.0, flatIndex: 2, source: "live" },
    { model: "google/gemini-flash-lite", vote: [0.03, 0.12, 0.45, 0.28, 0.12], weight: 1.0, flatIndex: 3, source: "live" },
  ];
  addVoteFrames(frames, TRIPLE_FS_VC, fsVotes);

  const fsAgg = aggregateScores(fsVotes);
  const fsScalar = scoresToScalar(fsAgg.scores);
  frames.push({ nodeId: TRIPLE_FS_VC, state: "resolved", scores: fsAgg.scores, output: fsScalar, label: fmt(fsScalar) });
  frames.push({ nodeId: TRIPLE_FS, state: "resolved", output: fsScalar, label: `five-star-rating → ${fmt(fsScalar)}` });

  frames.push({ nodeId: TRIPLE_SC, state: "pending", label: "sentiment-classifier" });
  frames.push({ nodeId: TRIPLE_SC_VC, state: "voting", label: "Positive / Negative / Neutral" });

  const scVotes: Vote[] = [
    { model: "deepseek/v3.2", vote: [0.10, 0.75, 0.15], weight: 1.0, flatIndex: 0, source: "live" },
    { model: "grok/4.1-fast", vote: [0.05, 0.85, 0.10], weight: 1.0, flatIndex: 1, source: "live" },
    { model: "openai/gpt-4o-mini", vote: [0.12, 0.70, 0.18], weight: 1.0, flatIndex: 2, source: "live" },
    { model: "google/gemini-flash-lite", vote: [0.08, 0.80, 0.12], weight: 1.0, flatIndex: 3, source: "live" },
  ];
  addVoteFrames(frames, TRIPLE_SC_VC, scVotes);

  const scAgg = aggregateScores(scVotes);
  const scScalar = scoresToScalar(scAgg.scores);
  frames.push({ nodeId: TRIPLE_SC_VC, state: "resolved", scores: scAgg.scores, output: scScalar, label: fmt(scScalar) });
  frames.push({ nodeId: TRIPLE_SC, state: "resolved", output: scScalar, label: `sentiment-classifier → ${fmt(scScalar)}` });

  const tripleScore = (bc2Scalar + fsScalar + scScalar) / 3;
  frames.push({ nodeId: TRIPLE, state: "resolved", output: tripleScore, label: `triple-classifier → ${fmt(tripleScore)}` });

  const rootScore = (spamScore + tripleScore) / 2;
  frames.push({ nodeId: ROOT, state: "resolved", output: rootScore, label: `Final: ${fmt(rootScore)}` });

  return { frames, agents: NANO_AGENTS };
}

function addVoteFrames(frames: ExecutionFrame[], nodeId: string, votes: Vote[]) {
  const accumulated: Vote[] = [];
  for (const vote of votes) {
    accumulated.push(vote);
    const agg = aggregateScores(accumulated);
    frames.push({
      nodeId,
      state: "voting",
      vote,
      scores: agg.scores,
      weights: agg.weights,
      label: `${vote.model.split("/").pop()} voted`,
    });
  }
}

function fmt(n: number): string {
  return n.toFixed(3);
}
