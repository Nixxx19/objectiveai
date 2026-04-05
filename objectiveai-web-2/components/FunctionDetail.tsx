"use client";

import { useState, useEffect, useMemo, useCallback } from "react";
import Link from "next/link";
import type {
  FunctionDefinition,
  TaskDefinition,
} from "@/lib/functions/types";
import type { FunctionDef, Task, TreeNode, TaskMeta } from "@/lib/tree/types";
import { fetchFunctionList, fetchFunctionDefinition } from "@/lib/functions/fetch";
import { apiFetch } from "@/lib/client";
import { buildTree } from "@/lib/tree/build";
import { simulateTree } from "@/lib/tree/simulateTree";
import { useExecution } from "@/lib/tree/useExecution";
import { FunctionTree } from "./FunctionTree";
import { ExecutionControls } from "./ExecutionControls";
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
  const [selectedNode, setSelectedNode] = useState<TreeNode | null>(null);
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

  const treeData = useMemo(() => {
    if (!rootDef) return null;
    return convertToTreeTypes(repo, owner, rootDef, allDefs);
  }, [rootDef, allDefs, repo, owner]);

  const timeline = useMemo(() => {
    if (!treeData) return null;
    const tree = buildTree(repo, treeData.root, treeData.registry);
    return simulateTree(tree);
  }, [treeData, repo]);

  const execution = useExecution(timeline ?? { frames: [], agents: [] });
  const [simulating, setSimulating] = useState(false);

  const handleNodeClick = useCallback((node: TreeNode) => {
    setSelectedNode((prev) => prev?.id === node.id ? null : node);
  }, []);

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

  if (!rootDef || !treeData) return null;

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

      {timeline && timeline.frames.length > 0 && (
        <div style={{ marginBottom: 12 }}>
          {!simulating ? (
            <button
              className={styles.schemaToggle}
              style={{ width: "auto", display: "inline-flex", border: "1px solid var(--node-border)", borderRadius: "var(--radius)" }}
              onClick={() => { setSimulating(true); execution.play(); }}
            >
              simulate execution
            </button>
          ) : (
            <ExecutionControls
              state={execution.state}
              onStep={execution.stepForward}
              onPlay={execution.play}
              onPause={execution.pause}
              onReset={() => { execution.reset(); setSimulating(false); }}
            />
          )}
        </div>
      )}

      <FunctionTree
        name={repo}
        root={treeData.root}
        registry={treeData.registry}
        executions={simulating ? execution.state.nodes : undefined}
        onNodeClick={handleNodeClick}
        selectedNodeId={selectedNode?.id}
      />

      {selectedNode?.taskMeta && (
        <NodeDetailPanel
          node={selectedNode}
          onClose={() => setSelectedNode(null)}
        />
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

/** Format an expression for display */
function formatExpr(expr: unknown): string {
  if (!expr) return "";
  if (typeof expr === "object" && expr !== null) {
    const e = expr as Record<string, unknown>;
    if (e.$starlark) return `$starlark: ${e.$starlark}`;
    if (e.$jmespath) return `$jmespath: ${e.$jmespath}`;
    return JSON.stringify(expr, null, 2);
  }
  return String(expr);
}

/** Extract message text from various formats */
function extractMessageText(content: unknown): string {
  if (typeof content === "string") return content;
  if (typeof content === "object" && content !== null) {
    const c = content as Record<string, unknown>;
    if (c.$starlark) return `\${starlark} ${c.$starlark}`;
    if (c.$jmespath) return `\${jmespath} ${c.$jmespath}`;
    return JSON.stringify(content);
  }
  return String(content);
}

function NodeDetailPanel({ node, onClose }: { node: TreeNode; onClose: () => void }) {
  const meta = node.taskMeta!;
  const isVote = node.kind === "vector-completion";

  return (
    <div className={styles.nodeDetail}>
      <div className={styles.nodeDetailHeader}>
        <span className={styles.nodeDetailTitle}>
          {isVote ? "vector.completion" : node.label}
          {node.functionType && !isVote && (
            <span style={{ color: "var(--info-dim)", marginLeft: 8, fontSize: 10 }}>
              {node.functionType}
            </span>
          )}
        </span>
        <button className={styles.nodeDetailClose} onClick={onClose}>
          close
        </button>
      </div>
      <div className={styles.nodeDetailBody}>
        {/* Messages (vote nodes) */}
        {isVote && meta.messages != null && (
          <div className={styles.nodeDetailSection}>
            <span className={styles.nodeDetailLabel}>messages</span>
            {Array.isArray(meta.messages) ? (
              (meta.messages as Array<{ role: string; content: unknown }>).map((msg, i) => (
                <div
                  key={i}
                  className={`${styles.nodeDetailMessage} ${
                    msg.role === "system" ? styles.nodeDetailSystem : styles.nodeDetailUser
                  }`}
                >
                  <div className={styles.nodeDetailRole}>{msg.role}</div>
                  {extractMessageText(msg.content)}
                </div>
              ))
            ) : (
              <div className={styles.nodeDetailCode}>
                {formatExpr(meta.messages)}
              </div>
            )}
          </div>
        )}

        {/* Full responses (vote nodes) */}
        {isVote && meta.fullResponses != null && meta.fullResponses.length > 0 ? (
          <div className={styles.nodeDetailSection}>
            <span className={styles.nodeDetailLabel}>
              responses ({meta.fullResponses.length})
            </span>
            {meta.fullResponses.map((r, i) => (
              <div key={i} className={styles.nodeDetailResponse}>{r}</div>
            ))}
          </div>
        ) : null}

        {/* Output expression */}
        {meta.outputExpr != null ? (
          <div className={styles.nodeDetailSection}>
            <span className={styles.nodeDetailLabel}>output</span>
            <div className={styles.nodeDetailCode}>
              {formatExpr(meta.outputExpr)}
            </div>
          </div>
        ) : null}

        {/* Input expression (function refs) */}
        {meta.inputExpr != null ? (
          <div className={styles.nodeDetailSection}>
            <span className={styles.nodeDetailLabel}>input</span>
            <div className={styles.nodeDetailCode}>
              {formatExpr(meta.inputExpr)}
            </div>
          </div>
        ) : null}

        {/* Input schema (function nodes) */}
        {meta.inputSchema != null && Object.keys(meta.inputSchema).length > 0 ? (
          <div className={styles.nodeDetailSection}>
            <span className={styles.nodeDetailLabel}>input_schema</span>
            <div className={styles.nodeDetailCode}>
              {JSON.stringify(meta.inputSchema, null, 2)}
            </div>
          </div>
        ) : null}

        {/* Description */}
        {node.description && (
          <div className={styles.nodeDetailSection}>
            <span className={styles.nodeDetailLabel}>description</span>
            <div style={{ fontFamily: "var(--font-sans)", fontSize: 11, color: "var(--info-dim)", lineHeight: 1.5 }}>
              {node.description}
            </div>
          </div>
        )}
      </div>
    </div>
  );
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

/** Convert API definitions to tree component types */
function convertToTreeTypes(
  rootRepo: string,
  rootOwner: string,
  rootDef: FunctionDefinition,
  allDefs: DefMap
): { root: FunctionDef; registry: Map<string, FunctionDef> } {
  const registry = new Map<string, FunctionDef>();

  for (const [key, def] of allDefs) {
    const repoName = key.split("/").pop()!;
    const converted: FunctionDef = {
      type: normalizeType(def.type) as FunctionDef["type"],
      description: def.description ?? "",
      input_schema: (def.input_schema ?? {}) as FunctionDef["input_schema"],
      tasks: def.tasks.map((t) => convertTask(t)),
    };
    registry.set(repoName, converted);
    registry.set(key, converted);
  }

  const root: FunctionDef = {
    type: normalizeType(rootDef.type) as FunctionDef["type"],
    description: rootDef.description ?? "",
    input_schema: (rootDef.input_schema ?? {}) as FunctionDef["input_schema"],
    tasks: rootDef.tasks.map((t) => convertTask(t)),
  };

  return { root, registry };
}

function convertTask(task: TaskDefinition): Task {
  if (task.type === "vector.completion") {
    return {
      type: "vector.completion",
      messages: task.messages,
      responses: task.responses ?? [],
      output: task.output,
      map: task.map,
    } as Task;
  }

  if (task.type.startsWith("placeholder.")) {
    return { type: task.type } as Task;
  }

  const name = task.repository ?? task.name ?? "unknown";
  const remote = task.owner ?? task.remote;

  return {
    type: normalizeType(task.type),
    name,
    remote,
    input: task.input,
    output: task.output,
    map: task.map,
  } as Task;
}

function normalizeType(type: string): string {
  let t = type;
  if (!t.startsWith("alpha.")) t = `alpha.${t}`;
  if (!t.endsWith(".function")) t = `${t}.function`;
  return t;
}
