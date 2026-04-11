"use client";

import { useState, useEffect, useMemo } from "react";
import Link from "next/link";
import type { FunctionDefinition } from "@/lib/functions/types";
import { fetchFunctionList, fetchFunctionDefinition, fetchRecursive } from "@/lib/functions/fetch";
import { apiFetch } from "@/lib/client";
import { fetchProfileBySlug } from "@/lib/profiles/fetch";
import type { ProfileMeta } from "@/lib/profiles/types";
import { adaptDefinition, adaptSubFunctions, adaptProfile } from "@/lib/tree/adapter";
import { FunctionTree } from "@objectiveai/function-tree";
import type { InputFunctionDefinition, InputFunctionExecution, InputProfile } from "@objectiveai/function-tree/core";
import { ExecutionResult } from "./ExecutionResult";
import "@objectiveai/function-tree/styles";
import "./FunctionTreeTheme.css";
import styles from "./FunctionCard.module.css";

// ── Example execution generation ──
// Generates deterministic, plausible execution data from a function definition
// and its paired profile. Used to show what running the function looks like.

function generateVote(numResponses: number, winnerIdx: number, confidence: number, seed: number): number[] {
  // Ensure winner always gets more than equal share
  const equalShare = 1 / numResponses;
  // Small per-model variance so rows aren't identical
  const jitter = ((seed * 7) % 10) / 100; // 0.00-0.09
  const winnerShare = equalShare + (confidence * 0.7 + jitter) * (1 - equalShare);
  const clamped = Math.min(winnerShare, 0.95);
  const remainder = 1 - clamped;
  return Array.from({ length: numResponses }, (_, i) =>
    i === winnerIdx ? clamped : remainder / Math.max(1, numResponses - 1)
  );
}

function buildExampleExecution(
  def: InputFunctionDefinition,
  profile: InputProfile,
  meta: ProfileMeta,
  slug: string,
): {
  execution: InputFunctionExecution;
  inputProfile: InputProfile;
  modelNames: Record<string, string>;
  responseLabels: Record<string, string[]>;
} | null {
  // Only generate for tasks that are vector completions (have votes)
  const vcTasks = def.tasks.filter((t) => t.type === "vector.completion");
  if (vcTasks.length === 0) return null;

  const modelNames: Record<string, string> = {};
  meta.llms.forEach((llm, i) => {
    modelNames[`llm-${String(i).padStart(3, "0")}`] = llm.model;
  });

  const responseLabels: Record<string, string[]> = {};

  const tasks = def.tasks.map((taskDef, ti) => {
    const isVc = taskDef.type === "vector.completion";
    const responses = (taskDef.responses ?? []) as string[];
    const numResponses = responses.length || 2;
    const profileTask = profile.tasks[ti];
    const weights = profileTask?.profile ?? meta.weights;
    const maxWeight = Math.max(...weights, 0);

    // Store response labels
    if (numResponses > 0) {
      responseLabels[String(ti)] = Array.from({ length: numResponses }, (_, ri) => {
        const r = responses[ri];
        if (typeof r === "string") return r.length > 20 ? r.slice(0, 18) + "…" : r;
        return `Response ${ri + 1}`;
      });
    }

    if (!isVc) {
      // Sub-function tasks: show as complete with a scalar score
      return {
        index: ti, task_index: ti, task_path: [ti],
        scores: [0.68], votes: [], completions: [],
      };
    }

    // Generate votes — each model votes with confidence proportional to its weight
    // Winner varies per task for visual variety
    const winnerIdx = ti % numResponses;
    const votes = weights.map((w, mi) => {
      const relW = maxWeight > 0 ? w / maxWeight : 0.5;
      // Low-weight models occasionally dissent (vote for next response)
      const thisWinner = relW < 0.3 && mi % 3 === 2
        ? (winnerIdx + 1) % numResponses
        : winnerIdx;
      return {
        model: `llm-${String(mi).padStart(3, "0")}`,
        vote: generateVote(numResponses, thisWinner, relW, mi + ti * 10),
        weight: w,
        from_cache: mi % 3 === 1, // Every third model cached
        from_rng: false,
      };
    });

    // Weighted scores
    const totalWeight = votes.reduce((s, v) => s + v.weight, 0);
    const scores = Array.from({ length: numResponses }, (_, ri) =>
      totalWeight > 0
        ? votes.reduce((s, v) => s + v.vote[ri] * v.weight, 0) / totalWeight
        : 1 / numResponses
    );

    return {
      index: ti, task_index: ti, task_path: [ti],
      scores, votes, completions: [],
    };
  });

  // Compute output based on function type
  const firstVc = tasks.find((t) => t.votes.length > 0);
  const isScalar = def.type === "scalar.function";
  // Scalar functions produce a single number; vector functions produce a score array.
  // Without evaluating output expressions, use the winning score as approximation.
  const output = isScalar
    ? (firstVc?.scores?.[0] ?? 0.5)
    : (firstVc?.scores ?? [1]);

  return {
    execution: {
      id: `example-${slug.slice(0, 12)}`,
      function: `${meta.owner}/${slug}`,
      profile: meta.repository,
      output,
      tasks,
    },
    inputProfile: profile,
    modelNames,
    responseLabels,
  };
}

interface Props {
  owner: string;
  repo: string;
}

type DefMap = Map<string, FunctionDefinition>;

export function FunctionDetail({ owner, repo }: Props) {
  const [rootDef, setRootDef] = useState<FunctionDefinition | null>(null);
  const [allDefs, setAllDefs] = useState<DefMap>(new Map());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [schemaOpen, setSchemaOpen] = useState(false);
  const [pairedProfile, setPairedProfile] = useState<ProfileMeta | null>(null);

  useEffect(() => {
    let cancelled = false;

    // Fetch paired profile — try pairs endpoint, fall back to profile-standard
    apiFetch<{ data: Array<{ function: { owner: string; repository: string }; profile: { owner: string; repository: string } }> }>("/functions/profiles/pairs")
      .then(async (pairs) => {
        if (cancelled) return;
        const match = pairs.data.find((p) => p.function.owner === owner && p.function.repository === repo);
        if (match) {
          const full = await fetchProfileBySlug(match.profile.owner, match.profile.repository);
          if (!cancelled) setPairedProfile(full);
        } else {
          // No pair found — use standard profile as default
          const fallback = await fetchProfileBySlug("ObjectiveAI", "profile-standard");
          if (!cancelled) setPairedProfile(fallback);
        }
      })
      .catch(async () => {
        // API unreachable — load standard profile directly from GitHub
        try {
          const fallback = await fetchProfileBySlug("ObjectiveAI", "profile-standard");
          if (!cancelled) setPairedProfile(fallback);
        } catch { /* truly offline */ }
      });

    fetchFunctionList()
      .then(async (items) => {
        const item = items.find(
          (f) => f.owner === owner && f.repository === repo
        );
        if (!item) throw new Error("Function not found");

        const commitMap = new Map<string, string>();
        for (const i of items) {
          commitMap.set(`${i.owner}/${i.repository}`, i.commit);
        }

        const defs = new Map<string, FunctionDefinition>();
        await fetchRecursive(item.owner, item.repository, item.commit, defs, commitMap);

        const root = defs.get(`${item.owner}/${item.repository}`);
        if (!root) throw new Error("Failed to fetch root definition");

        if (!cancelled) {
          setRootDef(root);
          setAllDefs(defs);
          setLoading(false);
        }
      })
      .catch((err) => {
        if (!cancelled) {
          setError(err.message);
          setLoading(false);
        }
      });

    return () => { cancelled = true; };
  }, [owner, repo]);

  const definition = useMemo(
    () => rootDef ? adaptDefinition(rootDef) : null,
    [rootDef]
  );

  const resolvedSubFunctions = useMemo(
    () => adaptSubFunctions(allDefs),
    [allDefs]
  );

  const inputProfile = useMemo(
    () => pairedProfile && rootDef ? adaptProfile(pairedProfile, rootDef.tasks.length) : null,
    [pairedProfile, rootDef]
  );

  const exampleExecution = useMemo(
    () => definition && inputProfile && pairedProfile
      ? buildExampleExecution(definition, inputProfile, pairedProfile, repo)
      : null,
    [definition, inputProfile, pairedProfile, repo]
  );

  if (loading) {
    return (
      <div className={styles.loading} role="status" aria-live="polite">
        <span className={styles.loadingDot} />
        loading {repo}
      </div>
    );
  }

  if (error) {
    return <div className={styles.error} role="alert">unable to load function</div>;
  }

  if (!rootDef || !definition) return null;

  const type = rootDef.type
    .replace(/^alpha\./, "")
    .replace(/\.function$/, "");

  const schema = rootDef.input_schema;
  const properties = (schema?.properties ?? {}) as Record<string, Record<string, unknown>>;
  const required = (schema?.required ?? []) as string[];
  const hasSchema = Object.keys(properties).length > 0;

  return (
    <div className={styles.detail}>
      <div className={styles.detailHeader}>
        <div className={styles.detailTitleRow}>
          <h1 className={styles.detailName}>{repo}</h1>
          <span className={styles.detailType}>{type}</span>
        </div>
        {rootDef.description && (
          <p className={styles.detailDescription}>{rootDef.description}</p>
        )}
        <p className={styles.detailOwner}>{owner}</p>

        {pairedProfile && (
          <div className={styles.profileCard}>
            <div className={styles.profileCardHeader}>
              <Link href="/profiles" className={styles.profileCardName}>
                {pairedProfile.name}
              </Link>
              <span className={styles.profileCardBadge}>{pairedProfile.totalAgents} agents</span>
            </div>
            {pairedProfile.description && (
              <p className={styles.profileCardDesc}>{pairedProfile.description}</p>
            )}
            {pairedProfile.weights.length > 0 && (
              <div className={styles.profileCardWeightBar}>
                {pairedProfile.weights.map((w, i) => {
                  const maxW = Math.max(...pairedProfile.weights);
                  const ratio = maxW > 0 ? w / maxW : 0;
                  return (
                    <div
                      key={i}
                      className={styles.profileCardWeightSeg}
                      style={{ flex: Math.max(w, 0.01), opacity: 0.2 + ratio * 0.8 }}
                    />
                  );
                })}
              </div>
            )}
            <div className={styles.profileCardTiers}>
              {pairedProfile.tiers.frontier.length > 0 && (
                <span className={styles.profileCardTier}>
                  <span className={styles.profileCardTierDot} data-tier="frontier" />
                  {pairedProfile.tiers.frontier.length} frontier
                </span>
              )}
              {pairedProfile.tiers.mid.length > 0 && (
                <span className={styles.profileCardTier}>
                  <span className={styles.profileCardTierDot} data-tier="mid" />
                  {pairedProfile.tiers.mid.length} mid
                </span>
              )}
              {pairedProfile.tiers.budget.length > 0 && (
                <span className={styles.profileCardTier}>
                  <span className={styles.profileCardTierDot} data-tier="budget" />
                  {pairedProfile.tiers.budget.length} budget
                </span>
              )}
            </div>
          </div>
        )}

        {hasSchema && (
          <div className={styles.schema}>
            <button
              className={styles.schemaToggle}
              onClick={() => setSchemaOpen(!schemaOpen)}
            >
              <span className={`${styles.schemaArrow} ${schemaOpen ? styles.schemaArrowOpen : ""}`}>
                ▸
              </span>
              input_schema
              {typeof schema?.description === "string" && (
                <span style={{ color: "var(--info-dim)", fontWeight: 400 }}>
                  — {schema.description}
                </span>
              )}
            </button>
            {schemaOpen && (
              <div className={styles.schemaBody}>
                {Object.entries(properties).map(([key, prop]) => (
                  <div key={key} className={styles.schemaProperty}>
                    <span className={styles.schemaKey}>{key}</span>
                    <span className={styles.schemaType}>
                      {renderSchemaType(prop)}
                    </span>
                    {required.includes(key) && (
                      <span className={styles.schemaRequired}>required</span>
                    )}
                    {typeof prop.description === "string" && (
                      <span className={styles.schemaDesc}>
                        {prop.description}
                      </span>
                    )}
                  </div>
                ))}
              </div>
            )}
          </div>
        )}
      </div>

      <FunctionTree
        data={null}
        definition={definition}
        resolvedSubFunctions={resolvedSubFunctions}
        height={500}
        borderless
        config={{ theme: "dark" }}
      />

      {exampleExecution && (
        <div style={{ marginTop: 16 }}>
          <p className={styles.nodeDetailLabel} style={{ marginBottom: 8 }}>
            example execution
          </p>
          <ExecutionResult
            execution={exampleExecution.execution}
            definition={definition}
            profile={exampleExecution.inputProfile}
            modelNames={exampleExecution.modelNames}
            responseLabels={exampleExecution.responseLabels}
          />
        </div>
      )}
    </div>
  );
}

/** Render schema type, handling anyOf and simple types */
function renderSchemaType(prop: Record<string, unknown>): string {
  if (prop.anyOf && Array.isArray(prop.anyOf)) {
    return (prop.anyOf as Record<string, unknown>[])
      .map((v) => (v.type as string) ?? "unknown")
      .join(" | ");
  }
  return (prop.type as string) ?? "object";
}

