"use client";

import Link from "next/link";
import type { ProfileMeta } from "@/lib/profiles/types";
import styles from "./ProfileCard.module.css";

/** Short model name: "openai/gpt-4o-mini" → "gpt-4o-mini" */
function shortModel(model: string): string {
  return model.includes("/") ? model.split("/").pop()! : model;
}

export function ProfileCard({ profile }: { profile: ProfileMeta }) {
  const isAuto = profile.kind === "auto";
  const totalLlms = isAuto
    ? profile.llms.length
    : profile.taskConfigs.reduce((sum, tc) => {
        const unique = new Set(tc.llms.map((l) => l.model));
        return sum + unique.size;
      }, 0);

  return (
    <div className={styles.card}>
      <div className={styles.cardBody}>
        <div className={styles.cardHeader}>
          <span className={styles.cardName}>{profile.name}</span>
          <span className={styles.cardKind}>{profile.kind}</span>
        </div>

        {profile.description && (
          <span className={styles.cardDescription}>{profile.description}</span>
        )}

        {/* Auto profile: single weight bar + agent chips */}
        {isAuto && profile.llms.length > 0 && (
          <>
            <div className={styles.weightBar}>
              {profile.weights.map((w, i) => (
                <div
                  key={i}
                  className={styles.weightSegment}
                  style={{ flex: Math.max(w, 0.01), opacity: 0.3 + (w / Math.max(...profile.weights)) * 0.7 }}
                />
              ))}
            </div>
            <div className={styles.agents}>
              {profile.llms.map((llm, i) => (
                <span key={i} className={styles.agentChip}>
                  {shortModel(llm.model)}
                  <span className={styles.agentWeight}>{profile.weights[i]?.toFixed(1)}</span>
                </span>
              ))}
            </div>
          </>
        )}

        {/* Tasks profile: per-task breakdown */}
        {!isAuto && profile.taskConfigs.length > 0 && (
          <div className={styles.taskSection}>
            <span className={styles.taskLabel}>
              {profile.taskConfigs.length} task{profile.taskConfigs.length !== 1 ? "s" : ""}
            </span>
            {profile.taskConfigs.map((tc, i) => {
              const taskWeight = profile.taskWeights[i] ?? 0;
              const uniqueModels = new Set(tc.llms.map((l) => l.model)).size;
              return (
                <div key={i} className={styles.taskRow}>
                  <span className={styles.taskIndex}>{i}</span>
                  <div className={styles.taskBar}>
                    {tc.weights.map((w, j) => (
                      <div
                        key={j}
                        className={styles.taskBarSegment}
                        style={{ flex: Math.max(w, 0.01), opacity: 0.3 + (w / Math.max(...tc.weights)) * 0.7 }}
                      />
                    ))}
                  </div>
                  <span className={styles.taskWeight}>{(taskWeight * 100).toFixed(0)}%</span>
                  <span className={styles.taskAgents}>{uniqueModels} agents</span>
                </div>
              );
            })}
          </div>
        )}

        {/* Paired function */}
        {profile.pairedFunction && (
          <span className={styles.pairedFunction}>
            powers{" "}
            <Link
              href={`/functions/${profile.pairedFunction.owner}/${profile.pairedFunction.repository}`}
              className={styles.pairedFunctionLink}
            >
              {profile.pairedFunction.repository}
            </Link>
          </span>
        )}

        <div className={styles.cardMeta}>
          <span className={styles.cardMetaItem}>
            <span className={styles.cardMetaDot} />
            {profile.owner}
          </span>
          {isAuto && (
            <span className={styles.cardMetaItem}>
              <span className={styles.cardMetaDot} />
              {totalLlms} agent{totalLlms !== 1 ? "s" : ""}
            </span>
          )}
        </div>
      </div>
    </div>
  );
}
