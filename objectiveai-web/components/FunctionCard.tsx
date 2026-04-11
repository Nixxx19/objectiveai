"use client";

import Link from "next/link";
import type { FunctionMeta } from "@/lib/functions/types";
import styles from "./FunctionCard.module.css";

export function FunctionCard({ fn }: { fn: FunctionMeta }) {
  const isBranch = fn.subFunctions.length > 0;

  return (
    <Link
      href={`/functions/${fn.owner}/${fn.repository}`}
      className={styles.card}
    >
      <div className={styles.cardBody}>
        <div className={styles.cardHeader}>
          <span className={styles.cardName}>{fn.name}</span>
          <span className={styles.cardType}>{fn.category}</span>
        </div>
        {fn.description && (
          <p className={styles.cardDescription}>{fn.description}</p>
        )}
        <div className={styles.cardMeta}>
          <span className={styles.cardMetaItem}>
            <span className={styles.cardMetaDot} />
            {fn.taskCount} task{fn.taskCount !== 1 ? "s" : ""}
          </span>
          {isBranch && (
            <span className={styles.cardMetaItem}>
              <span className={styles.cardMetaDot} />
              {fn.subFunctions.length} sub-function{fn.subFunctions.length !== 1 ? "s" : ""}
            </span>
          )}
        </div>
        {isBranch && (
          <div className={styles.cardSubFunctions}>
            {fn.subFunctions.map((name) => (
              <span key={name} className={styles.cardSubFunction}>{name}</span>
            ))}
          </div>
        )}
      </div>
    </Link>
  );
}
