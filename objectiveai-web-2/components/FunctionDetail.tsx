"use client";

import { useState, useEffect, useMemo } from "react";
import Link from "next/link";
import type { FunctionDefinition } from "@/lib/functions/types";
import { fetchFunctionList, fetchFunctionDefinition } from "@/lib/functions/fetch";
import { apiFetch } from "@/lib/client";
import { adaptDefinition, adaptSubFunctions } from "@/lib/tree/adapter";
import { FunctionTree } from "@objectiveai/function-tree";
import "@objectiveai/function-tree/styles";
import "./FunctionTreeTheme.css";
import styles from "./FunctionCard.module.css";

interface Props {
  owner: string;
  repo: string;
}

/** Resolved definition keyed by "owner/repository" */
type DefMap = Map<string, FunctionDefinition>;

export function FunctionDetail({ owner, repo }: Props) {
  const [rootDef, setRootDef] = useState<FunctionDefinition | null>(null);
  const [allDefs, setAllDefs] = useState<DefMap>(new Map());
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [schemaOpen, setSchemaOpen] = useState(false);
  const [pairedProfile, setPairedProfile] = useState<{ owner: string; repository: string } | null>(null);

  useEffect(() => {
    let cancelled = false;

    // Fetch paired profile in parallel (non-blocking)
    apiFetch<{ data: Array<{ function: { owner: string; repository: string }; profile: { owner: string; repository: string } }> }>("/functions/profiles/pairs")
      .then((pairs) => {
        if (cancelled) return;
        const match = pairs.data.find((p) => p.function.owner === owner && p.function.repository === repo);
        if (match) setPairedProfile(match.profile);
      })
      .catch(() => { /* non-critical */ });

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

  if (loading) {
    return (
      <div className={styles.loading} role="status" aria-live="polite">
        <span className={styles.loadingDot} />
        loading {repo}
      </div>
    );
  }

  if (error) {
    return <div className={styles.error} role="alert">{error}</div>;
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
        <p className={styles.detailOwner}>
          {owner}
          {pairedProfile && (
            <>
              {" · profile "}
              <Link
                href={`/profiles`}
                style={{ color: "var(--copper-dim)", textDecoration: "none" }}
              >
                {pairedProfile.repository}
              </Link>
            </>
          )}
        </p>

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

/** Recursively fetch a function and all its sub-function definitions */
async function fetchRecursive(
  owner: string,
  repository: string,
  commit: string,
  defs: DefMap,
  commitMap: Map<string, string>
): Promise<void> {
  const key = `${owner}/${repository}`;
  if (defs.has(key)) return;

  try {
    const def = await fetchFunctionDefinition(owner, repository, commit);
    defs.set(key, def);

    const subFetches = def.tasks
      .filter((t) => t.type.includes("function") && t.owner && t.repository)
      .map((t) => {
        const subCommit =
          t.commit ?? commitMap.get(`${t.owner}/${t.repository}`) ?? "main";
        return fetchRecursive(t.owner!, t.repository!, subCommit, defs, commitMap);
      });

    await Promise.allSettled(subFetches);
  } catch {
    // Silently skip functions we can't fetch
  }
}
