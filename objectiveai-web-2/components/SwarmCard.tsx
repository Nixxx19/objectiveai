"use client";

import type { SwarmMeta } from "@/lib/swarms/types";
import styles from "./SwarmCard.module.css";

/** Derive a readable label from the models in a swarm */
function swarmLabel(agents: SwarmMeta["agents"]): string {
  const models = agents.map((a) => {
    // "google/gemini-2.5-flash-lite" → "gemini-2.5-flash-lite"
    const short = a.model.includes("/") ? a.model.split("/").pop()! : a.model;
    // collapse long suffixes: "gpt-4o-mini" stays, "deepseek-chat-v3.2" → "deepseek-v3.2"
    return short.replace(/-chat(?=-)/i, "");
  });
  if (models.length <= 3) return models.join(" + ");
  return `${models.slice(0, 2).join(" + ")} +${models.length - 2}`;
}

export function SwarmCard({ swarm }: { swarm: SwarmMeta }) {
  const uniqueModels = new Set(swarm.agents.map((a) => a.model)).size;
  const label = swarmLabel(swarm.agents);

  return (
    <div className={styles.card}>
      <div className={styles.cardBody}>
        <div className={styles.cardHeader}>
          <span className={styles.cardName} title={swarm.id}>{label}</span>
          <span className={styles.cardCount}>
            {swarm.totalAgentCount} agent{swarm.totalAgentCount !== 1 ? "s" : ""}
          </span>
        </div>
        <span className={styles.cardId}>{swarm.id}</span>

        <div className={styles.agents}>
          {swarm.agents.map((agent) => (
            <div key={agent.id} className={styles.agent}>
              <span className={styles.agentModel}>{agent.model}</span>
              <span className={styles.agentMode}>{agent.outputMode}</span>
              <span className={styles.agentMeta}>
                {agent.topLogprobs != null && (
                  <span className={styles.agentDetail}>
                    logprobs:{agent.topLogprobs}
                  </span>
                )}
                {agent.temperature != null && (
                  <span className={styles.agentDetail}>
                    t:{agent.temperature}
                  </span>
                )}
                {agent.count > 1 && (
                  <span className={styles.agentCount}>×{agent.count}</span>
                )}
                {agent.hasFallbacks && (
                  <span className={styles.agentFallbacks}>
                    +{agent.fallbackCount} fallback{agent.fallbackCount !== 1 ? "s" : ""}
                  </span>
                )}
              </span>
            </div>
          ))}
        </div>

        <div className={styles.cardMeta}>
          <span className={styles.cardMetaItem}>
            <span className={styles.cardMetaDot} />
            {uniqueModels} model{uniqueModels !== 1 ? "s" : ""}
          </span>
          <span className={styles.cardMetaItem}>
            <span className={styles.cardMetaDot} />
            {swarm.agents.length} config{swarm.agents.length !== 1 ? "s" : ""}
          </span>
        </div>
      </div>
    </div>
  );
}
