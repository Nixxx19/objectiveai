"use client";

import { useState, useCallback, useEffect } from "react";
import Link from "next/link";
import PromptBlock from "@/components/PromptBlock";

export default function Home() {
  const [copied, setCopied] = useState(false);
  const [email, setEmail] = useState("");
  const [submitted, setSubmitted] = useState<false | "success" | "already">(
    false,
  );
  const [submitting, setSubmitting] = useState(false);

  const handleCopy = useCallback(async (text: string) => {
    try {
      await navigator.clipboard.writeText(text);
    } catch {
      const ta = document.createElement("textarea");
      ta.value = text;
      document.body.appendChild(ta);
      ta.select();
      document.execCommand("copy");
      document.body.removeChild(ta);
    }
    setCopied(true);
    setTimeout(() => setCopied(false), 1500);
  }, []);

  const handleEarlyAccess = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      if (!email || !email.includes("@")) return;
      setSubmitting(true);
      try {
        const res = await fetch("/api/early-access", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ email }),
        });
        const data = await res.json();
        if (data.already_subscribed) {
          setSubmitted("already");
        } else {
          setSubmitted("success");
        }
      } catch {
        setSubmitted("success");
      } finally {
        setSubmitting(false);
      }
    },
    [email],
  );

  useEffect(() => {
    const els = document.querySelectorAll(".landing-section, .landing-bottom-cta");
    const reveal = (el: Element) => {
      el.classList.add("visible");
    };
    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            reveal(entry.target);
            observer.unobserve(entry.target);
          }
        });
      },
      { threshold: 0.1 },
    );
    els.forEach((el) => {
      if (el.getBoundingClientRect().top < window.innerHeight) {
        reveal(el);
      } else {
        observer.observe(el);
      }
    });
    return () => observer.disconnect();
  }, []);

  return (
    <div className="landing">
      {/* ─── Hero ─── */}
      <section className="landing-hero">
        <div className="landing-badge">API + CLI live</div>

        <div className="landing-descriptor">
          Your agent&apos;s advisory board
        </div>

        <h1 className="landing-h1">
          Your agent doesn&apos;t
          <br />
          have to decide
          <br />
          alone.
        </h1>

        <p className="landing-sub">
          ObjectiveAI runs <strong>swarms of AI agents that vote</strong> on your
          inputs and return confidence-scored outputs. Not vibes &mdash;{" "}
          <strong>probability distributions</strong> from every agent&apos;s actual
          logprobs. One API call. Collective judgment.
        </p>

        {/* ── Primary CTA: Sign up ── */}
        <div className="landing-hero-ctas">
          <Link href="/api/auth/signin" className="landing-cta-btn primary">
            Sign up free
          </Link>
          <Link href="/functions" className="landing-cta-btn secondary">
            Browse functions
          </Link>
        </div>

        {/* ── Proof point ── */}
        <p className="landing-proof">
          <Link href="/functions" className="landing-proof-link">
            20+ scoring functions live and callable
          </Link>{" "}
          &mdash; invented by our agents.
        </p>

        {/* ── Terminal block ── */}
        <div className="landing-terminal">
          <div className="landing-terminal-bar">
            <span className="landing-dot" />
            <span className="landing-dot" />
            <span className="landing-dot" />
            <span className="landing-terminal-title">terminal</span>
          </div>
          <div className="landing-terminal-body">
            <button
              className={`landing-copy ${copied ? "landing-copy-copied" : ""}`}
              onClick={() =>
                handleCopy("npm install objectiveai")
              }
            >
              {copied ? "Copied!" : "Copy"}
            </button>
            <div className="landing-comment">
              # install the SDK
            </div>
            <div className="landing-line">
              <span className="landing-prompt">$</span>
              <span className="landing-cmd">npm install objectiveai</span>
            </div>
          </div>
        </div>
        <p className="landing-install-alt">
          Also available for <code>Rust</code>, <code>Go</code>, and <code>Python</code>
        </p>

        {/* ── Secondary CTA: Stay updated ── */}
        <div className="landing-cli-notify" id="early-access">
          <p className="landing-cli-notify-label">
            Stay in the loop
          </p>
          {!submitted ? (
            <form className="landing-ea-form" onSubmit={handleEarlyAccess}>
              <input
                type="email"
                placeholder="you@email.com"
                value={email}
                onChange={(e) => setEmail(e.target.value)}
              />
              <button type="submit" disabled={submitting}>
                {submitting ? "Sending..." : "Subscribe"}
              </button>
            </form>
          ) : submitted === "already" ? (
            <p className="landing-ea-confirmed">
              You&apos;re already on the list!
            </p>
          ) : (
            <p className="landing-ea-confirmed">
              You&apos;re in. Updates only when it matters.
            </p>
          )}
        </div>
      </section>

      {/* ─── The Problem / Key Insight ─── */}
      <section className="landing-section">
        <div className="landing-section-label">The Problem</div>
        <h2 className="landing-h2">One model, one opinion.</h2>
        <p className="landing-p">
          Every AI tool asks a single model for a single answer and
          throws away the uncertainty. You get a score, but no idea how confident
          it is.
        </p>
        <p className="landing-p">
          ObjectiveAI captures the{" "}
          <strong>full probability distribution</strong> from every agent in the
          swarm using logprobs &mdash; then combines them with learned
          weights.
        </p>

        <div className="landing-insight">
          <div className="landing-comparison">
            <div className="landing-side landing-side-old">
              <h4>Single Agent</h4>
              <pre>{`input → agent → "ship it"

// lost: agent was only 62% sure
// lost: 35% said "needs tests"
// lost: all nuance`}</pre>
            </div>
            <div className="landing-side landing-side-new">
              <h4>ObjectiveAI Swarm</h4>
              <pre>{`input → swarm → vote

// Agent 1:  [0.62, 0.35, 0.03]
// Agent 2:  [0.40, 0.55, 0.05]
// Agent 3:  [0.71, 0.20, 0.09]
// weighted: [0.58, 0.36, 0.06]`}</pre>
            </div>
          </div>
          <div className="landing-score-area">
            <div className="landing-score-label">
              Swarm result &mdash; &ldquo;Is this PR ready to merge?&rdquo;
            </div>
            <div className="landing-scores">
              {[
                { label: "ship it", pct: 58, value: "0.58", high: true },
                { label: "needs tests", pct: 36, value: "0.36", high: false },
                { label: "rethink", pct: 6, value: "0.06", high: false },
              ].map((s) => (
                <div key={s.label} className="landing-score-row">
                  <span className="landing-score-name">{s.label}</span>
                  <div className="landing-score-track">
                    <div
                      className={`landing-score-fill ${s.high ? "high" : "low"}`}
                      style={{ width: `${s.pct}%` }}
                    />
                  </div>
                  <span className="landing-score-val">{s.value}</span>
                </div>
              ))}
            </div>
          </div>
        </div>
      </section>

      {/* ─── How It Works ─── */}
      <section className="landing-section">
        <div className="landing-section-label">How It Works</div>
        <h2 className="landing-h2">Three concepts. That&apos;s it.</h2>

        <div className="landing-steps">
          {[
            {
              num: "1",
              title: "Swarms vote",
              desc: "Configure a swarm of agents \u2014 different models, tools, MCP servers, system prompts. Each agent votes independently using its full logprob distribution, not just a sampled answer.",
            },
            {
              num: "2",
              title: "Functions score",
              desc: (
                <>
                  A Function is a composable scoring pipeline: data in, score
                  out. Defined as a <code>function.json</code> on GitHub,
                  referenced by <code>owner/repo</code>. Agents can also invent functions
                  on their own &mdash; recursively.
                </>
              ),
            },
            {
              num: "3",
              title: "Profiles learn",
              desc: "Give ObjectiveAI a dataset of inputs and expected outputs. It optimizes the swarm weights to match \u2014 producing a Profile you can reuse. No fine-tuning. Just better voting.",
            },
          ].map((step) => (
            <div key={step.num} className="landing-step">
              <div className="landing-step-num">{step.num}</div>
              <div>
                <h3>{step.title}</h3>
                <p>{step.desc}</p>
              </div>
            </div>
          ))}
        </div>
      </section>

      {/* ─── Use Cases ─── */}
      <section className="landing-section">
        <div className="landing-section-label">Use Cases</div>
        <h2 className="landing-h2">Judgment where it matters.</h2>

        <div className="landing-usecases">
          {[
            {
              label: "Agent Judgment",
              desc: "Your agent is about to make a decision. A swarm evaluates it and returns a confidence-scored verdict before anything ships.",
            },
            {
              label: "Eval Pipelines",
              desc: "Score outputs with a swarm instead of a single judge. Confidence intervals, not coin flips.",
            },
            {
              label: "Content Moderation",
              desc: "Multi-agent consensus on safety. Know when agents disagree \u2014 that\u2019s where the edge cases live.",
            },
            {
              label: "Function Invention",
              desc: "Agents invent scoring functions recursively, spawning sub-agents to build sub-functions. Judgment that builds itself.",
            },
          ].map((uc) => (
            <div key={uc.label} className="landing-usecase">
              <div className="landing-uc-label">{uc.label}</div>
              {uc.desc}
            </div>
          ))}
        </div>
      </section>

      {/* ─── Browse ─── */}
      <section className="landing-section">
        <div className="landing-section-label">Browse</div>
        <h2 className="landing-h2">Functions invented by agents.</h2>
        <p className="landing-p">
          Every function is open, forkable, and callable via the API.
          See what agents have built.
        </p>
        <div className="landing-browse-link">
          <Link href="/functions" className="landing-cta-btn secondary">
            Browse functions &rarr;
          </Link>
        </div>
      </section>

      {/* ─── Bottom CTA ─── */}
      <section className="landing-bottom-cta">
        <h2 className="landing-bottom-cta-headline">
          Start building with ObjectiveAI
        </h2>
        <div className="landing-bottom-cta-block">
          <PromptBlock variant="compact" />
        </div>
        <div className="landing-cta-links">
          <Link href="/functions" className="landing-cta-btn secondary">
            Browse functions
          </Link>
        </div>
      </section>
    </div>
  );
}
