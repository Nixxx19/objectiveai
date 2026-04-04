"use client";

import { useState, useEffect, useRef, useCallback } from "react";
import Link from "next/link";

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface Vote {
  agent: string;
  model: string;
  scores: number[];
  revealed: boolean;
}

interface VotingRound {
  prompt: string;
  responses: string[];
  votes: Vote[];
  weights: number[];
  finalScores: number[];
}

// ---------------------------------------------------------------------------
// Data: real-looking voting scenarios
// ---------------------------------------------------------------------------

const SCENARIOS: Omit<VotingRound, "votes" | "finalScores">[] = [
  {
    prompt: "Is this PR ready to merge?",
    responses: ["ship it", "needs tests", "rethink"],
    weights: [0.40, 0.35, 0.25],
  },
  {
    prompt: "Rate content quality for publishing",
    responses: ["publish", "revise", "reject"],
    weights: [0.30, 0.45, 0.25],
  },
  {
    prompt: "Classify support ticket severity",
    responses: ["critical", "normal", "low"],
    weights: [0.50, 0.30, 0.20],
  },
  {
    prompt: "Evaluate candidate for role fit",
    responses: ["strong yes", "lean yes", "no"],
    weights: [0.35, 0.40, 0.25],
  },
];

const AGENT_POOL = [
  { agent: "agent-0", model: "gpt-4o" },
  { agent: "agent-1", model: "claude-sonnet-4-20250514" },
  { agent: "agent-2", model: "gemini-2.0-flash" },
  { agent: "agent-3", model: "llama-3.1-70b" },
  { agent: "agent-4", model: "mistral-large" },
];

function generateVotes(
  numResponses: number,
  agents: typeof AGENT_POOL
): Vote[] {
  return agents.slice(0, 3 + Math.floor(Math.random() * 2)).map((a) => {
    const raw = Array.from({ length: numResponses }, () => Math.random());
    // Make one score dominant for realism
    const dominant = Math.floor(Math.random() * numResponses);
    raw[dominant] *= 3;
    const sum = raw.reduce((s, v) => s + v, 0);
    const scores = raw.map((v) => Math.round((v / sum) * 100) / 100);
    // Fix rounding
    const diff = 1 - scores.reduce((s, v) => s + v, 0);
    scores[0] = Math.round((scores[0] + diff) * 100) / 100;
    return { ...a, scores, revealed: false };
  });
}

function computeFinalScores(votes: Vote[], weights: number[]): number[] {
  const numResponses = votes[0]?.scores.length ?? 0;
  const totals = new Array(numResponses).fill(0);
  let weightSum = 0;
  votes.forEach((v, i) => {
    const w = weights[i % weights.length];
    weightSum += w;
    v.scores.forEach((s, j) => {
      totals[j] += s * w;
    });
  });
  return totals.map((t) => Math.round((t / weightSum) * 100) / 100);
}

// ---------------------------------------------------------------------------
// Voting visualization component
// ---------------------------------------------------------------------------

function VotingTerminal() {
  const [round, setRound] = useState<VotingRound | null>(null);
  const [revealIndex, setRevealIndex] = useState(-1);
  const [showFinal, setShowFinal] = useState(false);
  const [cycleCount, setCycleCount] = useState(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const startRound = useCallback(() => {
    const scenario = SCENARIOS[cycleCount % SCENARIOS.length];
    const votes = generateVotes(scenario.responses.length, AGENT_POOL);
    const finalScores = computeFinalScores(votes, scenario.weights);
    setRound({ ...scenario, votes, finalScores });
    setRevealIndex(-1);
    setShowFinal(false);
  }, [cycleCount]);

  useEffect(() => {
    startRound();
  }, [startRound]);

  // Reveal votes one by one
  useEffect(() => {
    if (!round) return;
    if (revealIndex < round.votes.length) {
      timerRef.current = setTimeout(
        () => setRevealIndex((i) => i + 1),
        revealIndex === -1 ? 800 : 600
      );
    } else if (!showFinal) {
      timerRef.current = setTimeout(() => setShowFinal(true), 500);
    } else {
      // Pause on final, then cycle
      timerRef.current = setTimeout(() => {
        setCycleCount((c) => c + 1);
      }, 4000);
    }
    return () => {
      if (timerRef.current) clearTimeout(timerRef.current);
    };
  }, [round, revealIndex, showFinal]);

  if (!round) return null;

  return (
    <div className="lp-terminal">
      <div className="lp-terminal-chrome">
        <span className="lp-terminal-dot" />
        <span className="lp-terminal-title">vector completion</span>
      </div>
      <div className="lp-terminal-body">
        <div className="lp-terminal-line lp-terminal-prompt">
          <span className="lp-prompt-symbol">$</span>
          <span className="lp-prompt-cmd">objective</span>
          <span className="lp-prompt-arg">score</span>
          <span className="lp-prompt-str">
            &quot;{round.prompt}&quot;
          </span>
        </div>

        <div className="lp-terminal-line lp-terminal-dim">
          responses: [{round.responses.map((r, i) => (
            <span key={i}>
              {i > 0 && ", "}&quot;{r}&quot;
            </span>
          ))}]
        </div>

        <div className="lp-terminal-spacer" />

        {round.votes.map((vote, i) => (
          <div
            key={`${cycleCount}-${i}`}
            className={`lp-vote-row ${i < revealIndex ? "lp-vote-visible" : ""}`}
          >
            <span className="lp-vote-agent">{vote.model}</span>
            <span className="lp-vote-arrow">{"->"}</span>
            <span className="lp-vote-scores">
              [{vote.scores.map((s, j) => (
                <span key={j} className="lp-vote-score">
                  {j > 0 && " "}
                  <span
                    className={
                      s === Math.max(...vote.scores)
                        ? "lp-score-high"
                        : ""
                    }
                  >
                    {s.toFixed(2)}
                  </span>
                </span>
              ))}]
            </span>
          </div>
        ))}

        {showFinal && (
          <>
            <div className="lp-terminal-spacer" />
            <div className="lp-terminal-divider" />
            <div className="lp-final-row">
              <span className="lp-final-label">scores</span>
              <span className="lp-final-eq">=</span>
              <span className="lp-final-scores">
                [{round.finalScores.map((s, i) => (
                  <span key={i}>
                    {i > 0 && " "}
                    <span className="lp-final-value">{s.toFixed(2)}</span>
                  </span>
                ))}]
              </span>
            </div>
            <div className="lp-final-labels">
              {round.responses.map((r, i) => (
                <span key={i} className="lp-final-label-item">
                  {r}: {(round.finalScores[i] * 100).toFixed(0)}%
                </span>
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Install command with copy
// ---------------------------------------------------------------------------

function InstallCmd() {
  const [copied, setCopied] = useState(false);
  const cmd = "npm install objectiveai";

  const copy = async () => {
    try {
      await navigator.clipboard.writeText(cmd);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      /* noop */
    }
  };

  return (
    <button className="lp-install" onClick={copy} title="Copy to clipboard">
      <span className="lp-install-symbol">$</span>
      <span className="lp-install-text">{cmd}</span>
      <span className="lp-install-copy">{copied ? "copied" : "copy"}</span>
    </button>
  );
}

// ---------------------------------------------------------------------------
// Email capture
// ---------------------------------------------------------------------------

function EmailCapture() {
  const [email, setEmail] = useState("");
  const [state, setState] = useState<
    "idle" | "sending" | "done" | "exists" | "error"
  >("idle");

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email.trim()) return;
    setState("sending");
    try {
      const res = await fetch("/api/early-access", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ email: email.trim() }),
      });
      const data = await res.json();
      if (data.status === "already_subscribed") {
        setState("exists");
      } else if (res.ok) {
        setState("done");
      } else {
        setState("error");
      }
    } catch {
      setState("error");
    }
  };

  if (state === "done") {
    return <div className="lp-email-ok">noted. you are on the list.</div>;
  }
  if (state === "exists") {
    return <div className="lp-email-ok">already subscribed.</div>;
  }

  return (
    <form className="lp-email-form" onSubmit={submit}>
      <input
        type="email"
        className="lp-email-input"
        placeholder="email for updates"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
        disabled={state === "sending"}
      />
      <button
        type="submit"
        className="lp-email-btn"
        disabled={state === "sending" || !email.trim()}
      >
        {state === "sending" ? "..." : "notify me"}
      </button>
      {state === "error" && (
        <span className="lp-email-err">something went wrong</span>
      )}
    </form>
  );
}

// ---------------------------------------------------------------------------
// Page
// ---------------------------------------------------------------------------

export default function Home() {
  return (
    <div className="landing">
      <div className="lp-root">
        {/* The machine. This IS the page. */}
        <section className="lp-stage">
          <div className="lp-stage-left">
            <div className="lp-identity">
              <h1 className="lp-wordmark">ObjectiveAI</h1>
              <p className="lp-descriptor">
                Ensemble LLM scoring pipelines as an API
              </p>
            </div>

            <div className="lp-prose">
              <p>
                Multiple models vote with weighted probabilities.
                Votes converge into structured numeric scores.
                Not text. Not chat. Numbers you can trust.
              </p>
            </div>

            <div className="lp-actions">
              <InstallCmd />
              <div className="lp-action-links">
                <Link href="/api/auth/signin" className="lp-link-primary">
                  sign in
                </Link>
                <Link href="/functions" className="lp-link-secondary">
                  browse functions
                </Link>
              </div>
            </div>

            <div className="lp-email-section">
              <EmailCapture />
            </div>
          </div>

          <div className="lp-stage-right">
            <VotingTerminal />
          </div>
        </section>

        {/* Concept strip: the vocabulary of the system */}
        <section className="lp-concepts">
          <div className="lp-concept">
            <span className="lp-concept-name">swarm</span>
            <span className="lp-concept-desc">
              a group of agents — different models, prompts, tools — that vote independently
            </span>
          </div>
          <div className="lp-concept">
            <span className="lp-concept-name">vector completion</span>
            <span className="lp-concept-desc">
              the core primitive. produces score vectors, not text. each agent votes, weights combine, returns a probability distribution
            </span>
          </div>
          <div className="lp-concept">
            <span className="lp-concept-name">function</span>
            <span className="lp-concept-desc">
              a composable scoring pipeline. data in, score out. hosted on GitHub as function.json
            </span>
          </div>
          <div className="lp-concept">
            <span className="lp-concept-name">profile</span>
            <span className="lp-concept-desc">
              learned weights that optimize how a swarm votes. trained, not configured
            </span>
          </div>
        </section>

        {/* The API call. Show the code. */}
        <section className="lp-code-section">
          <div className="lp-code-block">
            <div className="lp-code-header">
              <span className="lp-code-lang">typescript</span>
            </div>
            <pre className="lp-code-pre"><code>{`import ObjectiveAI from "objectiveai";

const client = new ObjectiveAI({ apiKey });

const result = await client.functions.executions.create(
  "your-org/content-quality",
  {
    input: { text: document.body },
    profile: "production",
  }
);

// result.output.scores
// → [0.58, 0.36, 0.06]
// → { publish: 0.58, revise: 0.36, reject: 0.06 }`}</code></pre>
          </div>
        </section>

        <footer className="lp-footer">
          <span className="lp-footer-text">
            api.objective-ai.io
          </span>
        </footer>
      </div>
    </div>
  );
}
