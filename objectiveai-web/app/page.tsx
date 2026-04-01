"use client";

import { useState, useCallback, useEffect } from "react";
import Link from "next/link";
import PromptBlock from "@/components/PromptBlock";

/**
 * Flip to `true` when the CLI skill ships.
 * Controls: terminal block switches from "coming soon" preview to real commands.
 */
const CLI_LIVE = false;

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
        <div className="landing-badge">Live now &mdash; CLI coming soon</div>

        <div className="landing-descriptor">
          An agentic collective judgment harness
        </div>

        <h1 className="landing-h1">
          Score everything.
          <br />
          Trust nothing
          <br />
          to one model.
        </h1>

        <p className="landing-sub">
          ObjectiveAI runs <strong>ensembles of LLMs that vote</strong> on your
          inputs and return confidence-scored outputs. Not vibes &mdash;{" "}
          <strong>probability distributions</strong> from the models&apos; actual
          logprobs. One function call. Real signal.
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
          &mdash; built by our autonomous agent.
        </p>

        {/* ── Terminal block ── */}
        <div className="landing-terminal">
          <div className="landing-terminal-bar">
            <span className="landing-dot" />
            <span className="landing-dot" />
            <span className="landing-dot" />
            <span className="landing-terminal-title">
              {CLI_LIVE ? "terminal" : "terminal"}
            </span>
          </div>
          <div className="landing-terminal-body">
            {CLI_LIVE ? (
              <>
                <button
                  className={`landing-copy ${copied ? "landing-copy-copied" : ""}`}
                  onClick={() =>
                    handleCopy(
                      "npm install objectiveai\nobjectiveai install-skill",
                    )
                  }
                >
                  {copied ? "Copied!" : "Copy"}
                </button>
                <div className="landing-comment">
                  # install the SDK and attach to Claude Code
                </div>
                <div className="landing-line">
                  <span className="landing-prompt">$</span>
                  <span className="landing-cmd">npm install objectiveai</span>
                </div>
                <div className="landing-line">
                  <span className="landing-prompt">$</span>
                  <span className="landing-cmd">
                    objectiveai install-skill
                  </span>
                </div>
              </>
            ) : (
              <>
                <button
                  className={`landing-copy ${copied ? "landing-copy-copied" : ""}`}
                  onClick={() => handleCopy("npm install objectiveai")}
                >
                  {copied ? "Copied!" : "Copy"}
                </button>
                <div className="landing-comment">
                  # install the SDK (works today)
                </div>
                <div className="landing-line">
                  <span className="landing-prompt">$</span>
                  <span className="landing-cmd">npm install objectiveai</span>
                </div>
                <br />
                <div className="landing-comment">
                  # attach to Claude Code (coming soon)
                </div>
                <div className="landing-line">
                  <span className="landing-prompt">$</span>
                  <span className="landing-cmd landing-cmd-dim">
                    objectiveai install-skill
                  </span>
                </div>
              </>
            )}
          </div>
        </div>
        <p className="landing-install-alt">
          Also available as <code>cargo add objectiveai</code>
        </p>

        {/* ── Secondary CTA: CLI notification ── */}
        <div className="landing-cli-notify" id="early-access">
          <p className="landing-cli-notify-label">
            Get notified when the CLI ships
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
                {submitting ? "Sending..." : "Notify me"}
              </button>
            </form>
          ) : submitted === "already" ? (
            <p className="landing-ea-confirmed">
              You&apos;re already on the list!
            </p>
          ) : (
            <p className="landing-ea-confirmed">
              You&apos;re on the list. One email when it ships.
            </p>
          )}
        </div>
      </section>

      {/* ─── The Problem / Key Insight ─── */}
      <section className="landing-section">
        <div className="landing-section-label">The Problem</div>
        <h2 className="landing-h2">One model, one opinion.</h2>
        <p className="landing-p">
          Every LLM scoring tool asks a single model for a single answer and
          throws away the uncertainty. You get a score, but no idea how confident
          it is.
        </p>
        <p className="landing-p">
          ObjectiveAI captures the{" "}
          <strong>full probability distribution</strong> from every model in the
          ensemble using logprobs &mdash; then combines them with learned
          weights.
        </p>

        <div className="landing-insight">
          <div className="landing-comparison">
            <div className="landing-side landing-side-old">
              <h4>Typical LLM Scoring</h4>
              <pre>{`prompt → model → "A"

// lost: model was only 70% sure
// lost: 30% signal for "B"
// lost: all nuance`}</pre>
            </div>
            <div className="landing-side landing-side-new">
              <h4>ObjectiveAI</h4>
              <pre>{`prompt → ensemble → vote

// GPT-4o:   [0.70, 0.30, 0.00]
// Claude:   [0.55, 0.40, 0.05]
// Llama:    [0.80, 0.15, 0.05]
// weighted: [0.69, 0.28, 0.03]`}</pre>
            </div>
          </div>
          <div className="landing-score-area">
            <div className="landing-score-label">
              Ensemble result &mdash; &ldquo;What color is the sky?&rdquo;
            </div>
            <div className="landing-scores">
              {[
                { label: "blue", pct: 85, value: "0.85", high: true },
                { label: "gray", pct: 8, value: "0.08", high: false },
                { label: "red", pct: 4, value: "0.04", high: false },
                { label: "green", pct: 3, value: "0.03", high: false },
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
              title: "Ensembles vote",
              desc: "Configure a group of LLMs \u2014 different models, temperatures, system prompts. Each one votes independently on your input using its full logprob distribution, not just a sampled answer.",
            },
            {
              num: "2",
              title: "Functions score",
              desc: (
                <>
                  A Function is a composable scoring pipeline: data in, score
                  out. Defined as a <code>function.json</code> on GitHub.
                  Reference by <code>owner/repo</code>. One line to call, fully
                  versioned.
                </>
              ),
            },
            {
              num: "3",
              title: "Profiles learn",
              desc: "Give ObjectiveAI a dataset of inputs and expected outputs. It optimizes the ensemble weights to match \u2014 producing a Profile you can reuse. No fine-tuning. Just better voting.",
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
        <h2 className="landing-h2">If it can be scored, rank it.</h2>

        <div className="landing-usecases">
          {[
            {
              label: "Eval Pipelines",
              desc: "Score LLM outputs with an ensemble instead of a single judge model. Get confidence intervals, not coin flips.",
            },
            {
              label: "Content Moderation",
              desc: "Multi-model consensus on safety classifications. Know when models disagree \u2014 that\u2019s where the edge cases live.",
            },
            {
              label: "Search Ranking",
              desc: "Re-rank search results, candidates, or recommendations with weighted ensemble scoring.",
            },
            {
              label: "Simulate Perspectives",
              desc: "Give each LLM a different persona. See how a skeptic, an optimist, and a domain expert would each score the same input.",
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
        <h2 className="landing-h2">Functions built by our agent.</h2>
        <p className="landing-p">
          Every function is open, forkable, and callable via the API.
          Explore what&apos;s already been scored.
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
