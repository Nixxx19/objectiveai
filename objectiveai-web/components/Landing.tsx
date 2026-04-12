"use client";

import { useState, useEffect, useMemo } from "react";
import Link from "next/link";
import type { FunctionMeta } from "@/lib/functions/types";
import type { SwarmMeta } from "@/lib/swarms/types";
import type { ProfileMeta } from "@/lib/profiles/types";
import { fetchAllFunctions } from "@/lib/functions/fetch";
import { fetchAllSwarms } from "@/lib/swarms/fetch";
import { fetchDefaultProfiles } from "@/lib/profiles/fetch";
import styles from "./Landing.module.css";

/** Static example — realistic votes from a startup-idea-ranker execution */
const EXAMPLE_VOTES = [
  { model: "claude-haiku-4.5",        vote: [0.00, 1.00], weight: 1.0 },
  { model: "gpt-4.1-nano",            vote: [0.85, 0.15], weight: 1.0 },
  { model: "gpt-4.1-nano",            vote: [0.70, 0.30], weight: 1.0 },
  { model: "gemini-2.5-flash-lite",   vote: [0.60, 0.40], weight: 0.8 },
  { model: "gemini-2.5-flash-lite",   vote: [0.45, 0.55], weight: 0.8 },
  { model: "gemini-3-flash-preview",  vote: [0.55, 0.45], weight: 0.8 },
  { model: "gpt-5-mini",              vote: [0.90, 0.10], weight: 1.0 },
  { model: "grok-4.1-fast",           vote: [0.40, 0.60], weight: 0.5 },
  { model: "grok-4.1-fast",           vote: [0.50, 0.50], weight: 0.5 },
  { model: "deepseek-v3.2",           vote: [0.75, 0.25], weight: 0.3 },
  { model: "deepseek-v3.2",           vote: [0.60, 0.40], weight: 0.3 },
  { model: "gpt-4o-mini",             vote: [0.55, 0.45], weight: 0.5 },
];

export function Landing() {
  const [functions, setFunctions] = useState<FunctionMeta[]>([]);
  const [swarms, setSwarms] = useState<SwarmMeta[]>([]);
  const [profiles, setProfiles] = useState<ProfileMeta[]>([]);
  const [loaded, setLoaded] = useState({ fn: false, sw: false, pr: false });
  const [errors, setErrors] = useState({ fn: false, sw: false, pr: false });

  useEffect(() => {
    let cancelled = false;

    fetchAllFunctions()
      .then((fns) => { if (!cancelled) { setFunctions(fns); setLoaded((p) => ({ ...p, fn: true })); } })
      .catch(() => { if (!cancelled) { setErrors((p) => ({ ...p, fn: true })); setLoaded((p) => ({ ...p, fn: true })); } });

    fetchAllSwarms()
      .then((s) => { if (!cancelled) { setSwarms(s); setLoaded((p) => ({ ...p, sw: true })); } })
      .catch(() => { if (!cancelled) { setErrors((p) => ({ ...p, sw: true })); setLoaded((p) => ({ ...p, sw: true })); } });

    fetchDefaultProfiles()
      .then((p) => { if (!cancelled) { setProfiles(p); setLoaded((prev) => ({ ...prev, pr: true })); } })
      .catch(() => { if (!cancelled) { setErrors((p) => ({ ...p, pr: true })); setLoaded((p) => ({ ...p, pr: true })); } });

    return () => { cancelled = true; };
  }, []);

  const sortedFunctions = useMemo(() => {
    return [...functions].sort((a, b) => {
      const aB = a.subFunctions.length > 0 ? 0 : 1;
      const bB = b.subFunctions.length > 0 ? 0 : 1;
      if (aB !== bB) return aB - bB;
      return a.name.localeCompare(b.name);
    });
  }, [functions]);

  const sortedSwarms = useMemo(() => {
    return [...swarms].sort((a, b) => b.totalAgentCount - a.totalAgentCount);
  }, [swarms]);

  return (
    <div className={styles.landing}>
      {/* Hero */}
      <section className={styles.hero}>
        <h1 className={styles.heroTitle}>the agentic collective judgment harness</h1>
        <div className={styles.install}>
          <span className={styles.installCmd}>npm install objectiveai</span>
          <span className={styles.installCmd}>pip install objectiveai</span>
        </div>
      </section>

      <p className={styles.bridging}>your agent doesn't have to decide alone</p>

      {/* Execution comparison */}
      <section className={styles.execution}>
        <div className={styles.executionHeader}>
          <span className={styles.executionLabel}>example execution</span>
          <span className={styles.executionFunction}>startup-idea-ranker</span>
        </div>
        <div className={styles.responseOptions}>
          <span className={styles.responseOption}>
            <span className={styles.responseMarker} data-option="a" />A: AI tutoring platform
          </span>
          <span className={styles.responseOption}>
            <span className={styles.responseMarker} data-option="b" />B: blockchain pet insurance
          </span>
        </div>

        <div className={styles.comparisonPanels}>
          {/* Left — single agent */}
          <div className={styles.panel}>
            <div className={styles.panelHeader}>
              <span className={styles.panelTitle}>one agent</span>
            </div>
            <div className={styles.voteTable}>
              <div className={styles.voteTableHeader}>
                <span className={styles.voteTableCol}>agent</span>
                <span className={styles.voteTableColBar}>vote</span>
                <span className={styles.voteTableCol}>weight</span>
              </div>
              <div className={styles.agentRow}>
                <span className={styles.agentModel}>{EXAMPLE_VOTES[0].model}</span>
                <div className={styles.agentVoteBar}>
                  <div className={styles.voteSegmentA} style={{ flex: Math.max(EXAMPLE_VOTES[0].vote[0], 0.02) }} />
                  <div className={styles.voteSegmentB} style={{ flex: Math.max(EXAMPLE_VOTES[0].vote[1], 0.02) }} />
                </div>
                <span className={styles.agentWeight}>{EXAMPLE_VOTES[0].weight.toFixed(1)}</span>
              </div>
            </div>
            <div className={styles.scoresRow}>
              <span className={styles.scoresLabel}>scores</span>
              <div className={styles.scoreBar}>
                <div className={styles.scoreSegmentA} style={{ flex: 0.02 }} />
                <div className={styles.scoreSegmentB} style={{ flex: 0.98 }} />
              </div>
              <span className={styles.scoresValue}>[0.00, 1.00]</span>
            </div>
          </div>

          {/* Right — full swarm */}
          <div className={styles.panel}>
            <div className={styles.panelHeader}>
              <span className={styles.panelTitle}>the swarm</span>
              <span className={styles.panelMeta}>profile-giga · 12 agents</span>
            </div>
            <div className={styles.voteTable}>
              <div className={styles.voteTableHeader}>
                <span className={styles.voteTableCol}>agent</span>
                <span className={styles.voteTableColBar}>vote distribution</span>
                <span className={styles.voteTableCol}>weight</span>
              </div>
              {EXAMPLE_VOTES.map((v, i) => {
                const relWeight = v.weight; // max weight is 1.0
                return (
                  <div
                    key={i}
                    className={`${styles.agentRow} ${i === 0 ? styles.agentHighlight : ""}`}
                    style={relWeight >= 0.8 ? { background: `rgba(217, 119, 6, ${relWeight * 0.04})` } : undefined}
                  >
                    <span className={styles.agentModel}>{v.model}</span>
                    <div className={styles.agentVoteBar}>
                      <div className={styles.voteSegmentA} style={{ flex: Math.max(v.vote[0], 0.02) }} />
                      <div className={styles.voteSegmentB} style={{ flex: Math.max(v.vote[1], 0.02) }} />
                    </div>
                    <div className={styles.weightFader}>
                      <div className={styles.weightFaderTrack}>
                        <div
                          className={styles.weightFaderFill}
                          style={{
                            height: `${relWeight * 100}%`,
                            background: `rgba(217, 119, 6, ${0.3 + relWeight * 0.5})`,
                          }}
                        />
                      </div>
                      <span className={styles.agentWeight}>{v.weight.toFixed(1)}</span>
                    </div>
                  </div>
                );
              })}
            </div>
            <div className={styles.scoresRow}>
              <span className={styles.scoresLabel}>scores</span>
              <div className={styles.scoreBar}>
                <div className={styles.scoreSegmentA} style={{ flex: 0.62 }} />
                <div className={styles.scoreSegmentB} style={{ flex: 0.38 }} />
              </div>
              <span className={styles.scoresValue}>[0.62, 0.38]</span>
            </div>
          </div>
        </div>
      </section>

      {/* Directory */}
      <section className={styles.directory}>
        {/* Functions */}
        <div className={styles.directorySection}>
          <div className={styles.sectionHeader}>
            <h2 className={styles.sectionTitle}>functions</h2>
            <Link href="/functions" className={styles.sectionLink}>browse all</Link>
          </div>
          {!loaded.fn ? <SectionLoader /> : sortedFunctions.length === 0 ? (
            <div className={styles.sectionEmpty}>functions are registered via the api — check github for examples</div>
          ) : (
            <div className={styles.directoryList}>
              {sortedFunctions.slice(0, 6).map((fn) => (
                <Link
                  key={`${fn.owner}/${fn.repository}`}
                  href={`/functions/${fn.owner}/${fn.repository}`}
                  className={styles.directoryRow}
                >
                  <span className={styles.directoryName}>{fn.name}</span>
                  <span className={styles.directoryMeta}>{fn.type}</span>
                  <span className={styles.directoryMeta}>{fn.taskCount} tasks</span>
                  {fn.subFunctions.length > 0 && (
                    <span className={styles.directoryMeta}>{fn.subFunctions.length} sub-functions</span>
                  )}
                </Link>
              ))}
            </div>
          )}
        </div>

        {/* Swarms */}
        <div className={styles.directorySection}>
          <div className={styles.sectionHeader}>
            <h2 className={styles.sectionTitle}>swarms</h2>
            <Link href="/swarms" className={styles.sectionLink}>browse all</Link>
          </div>
          {!loaded.sw ? <SectionLoader /> : sortedSwarms.length === 0 ? (
            <div className={styles.sectionEmpty}>swarms are created at execution time</div>
          ) : (
            <div className={styles.directoryList}>
              {sortedSwarms.slice(0, 4).map((s) => (
                <Link
                  key={s.id}
                  href={`/swarms/${s.id}`}
                  className={styles.directoryRow}
                >
                  <span className={styles.directoryName}>{swarmSummary(s)}</span>
                  <span className={styles.directoryMeta}>{s.totalAgentCount} agents</span>
                </Link>
              ))}
            </div>
          )}
        </div>

        {/* Profiles */}
        <div className={styles.directorySection}>
          <div className={styles.sectionHeader}>
            <h2 className={styles.sectionTitle}>profiles</h2>
            <Link href="/profiles" className={styles.sectionLink}>browse all</Link>
          </div>
          {!loaded.pr ? <SectionLoader /> : profiles.length === 0 ? (
            <div className={styles.sectionEmpty}>profiles are loaded from github</div>
          ) : (
            <div className={styles.directoryList}>
              {profiles.map((p) => (
                <div key={p.name} className={styles.directoryRow}>
                  <span className={styles.directoryName}>{p.name}</span>
                  <span className={styles.directoryMeta}>{p.totalAgents} agents</span>
                </div>
              ))}
            </div>
          )}
        </div>
      </section>
    </div>
  );
}

/** Derive a short label from a swarm's agent models */
function swarmSummary(s: SwarmMeta): string {
  const unique = [...new Set(s.agents.map((a) => {
    const short = a.model.includes("/") ? a.model.split("/").pop()! : a.model;
    return short.replace(/-chat(?=-)/i, "");
  }))];
  if (unique.length <= 2) return unique.join(" + ");
  return `${unique[0]} + ${unique[1]} +${unique.length - 2}`;
}

function SectionLoader() {
  return (
    <div className={styles.sectionLoading}>
      <span className={styles.loadingDot} />
    </div>
  );
}
