import type {
  FunctionsExecutionsResponseStreamingFunctionExecutionChunk,
  FunctionsExecutionsResponseStreamingTaskChunk,
  FunctionsExecutionsResponseStreamingVectorCompletionTaskChunk,
  VectorCompletionsResponseStreamingAgentCompletionChunk,
} from "objectiveai";
import { AgentCompletionChat } from "./AgentCompletionView";

interface FunctionExecutionEntry {
  kind: "execution";
  id: string;
  request: { id: string };
  chunk: FunctionsExecutionsResponseStreamingFunctionExecutionChunk | null;
  error: { id: string; code: number; message: unknown } | null;
}

/// One leaf entry to render: a vector completion inside the execution tree,
/// identified by a `task_path` plus its sequence of agent completions.
interface VectorLeaf {
  task_path: number[];
  /// Optional modifiers ([split_index=.., swiss_pool_index=.., swiss_round=..])
  /// pulled from the parent function-execution task wrappers.
  modifiers: string[];
  vector: FunctionsExecutionsResponseStreamingVectorCompletionTaskChunk;
}

// `TaskChunk` is `#[serde(untagged)]` in Rust, so the JSON has no
// `{"FunctionExecution": …}` discriminator — both variants serialize as
// flat objects whose fields come from their inner struct (via `#[flatten]`).
// We discriminate by the `object` marker that every inner chunk carries.
function isVectorCompletionTask(t: unknown): boolean {
  return (
    typeof t === "object" &&
    t !== null &&
    "object" in t &&
    (t as { object?: unknown }).object === "vector.completion.chunk"
  );
}

function isFunctionExecutionTask(t: unknown): boolean {
  if (typeof t !== "object" || t === null || !("object" in t)) return false;
  const o = (t as { object?: unknown }).object;
  return (
    o === "scalar.function.execution.chunk" ||
    o === "vector.function.execution.chunk"
  );
}

function collectVectorLeaves(
  tasks: FunctionsExecutionsResponseStreamingTaskChunk[] | undefined,
  inheritedModifiers: string[],
  out: VectorLeaf[],
): void {
  if (!tasks) return;
  for (const t of tasks) {
    if (isVectorCompletionTask(t)) {
      const v = t as unknown as FunctionsExecutionsResponseStreamingVectorCompletionTaskChunk;
      out.push({ task_path: v.task_path ?? [], modifiers: inheritedModifiers, vector: v });
      continue;
    }
    if (isFunctionExecutionTask(t)) {
      // Function-execution wrapper: pick up any split/swiss modifiers, then
      // recurse into its (flattened) `tasks` array.
      const fe = t as unknown as {
        split_index?: number | null;
        swiss_pool_index?: number | null;
        swiss_round?: number | null;
        tasks?: FunctionsExecutionsResponseStreamingTaskChunk[];
      };
      const mods = [...inheritedModifiers];
      if (fe.split_index !== undefined && fe.split_index !== null) {
        mods.push(`split=${fe.split_index}`);
      }
      if (fe.swiss_pool_index !== undefined && fe.swiss_pool_index !== null) {
        mods.push(`pool=${fe.swiss_pool_index}`);
      }
      if (fe.swiss_round !== undefined && fe.swiss_round !== null) {
        mods.push(`round=${fe.swiss_round}`);
      }
      collectVectorLeaves(fe.tasks, mods, out);
      continue;
    }
  }
}

function formatTaskPath(path: number[], modifiers: string[]): string {
  const p = path.length === 0 ? "(root)" : path.join(".");
  return modifiers.length === 0 ? p : `${p}  [${modifiers.join(", ")}]`;
}

export function FunctionExecutionView({ entry }: { entry: FunctionExecutionEntry }) {
  const chunk = entry.chunk;
  const topError = entry.error
    ? { code: entry.error.code, message: entry.error.message }
    : null;

  const leaves: VectorLeaf[] = [];
  if (chunk) {
    collectVectorLeaves(chunk.tasks, [], leaves);
  }

  return (
    <div>
      {leaves.length > 0 &&
        leaves.map((leaf, leafIdx) => {
          const pathLabel = formatTaskPath(leaf.task_path, leaf.modifiers);
          const vec = leaf.vector;
          const vecError = vec.error
            ? { code: vec.error.code, message: vec.error.message }
            : null;

          return (
            <div key={`${pathLabel}-${leafIdx}`}>
              <div className="ac-section-header">{pathLabel}</div>

              {(vec.completions?.length ?? 0) === 0 && !vecError && (
                <div style={{ maxWidth: 800, margin: "0 auto 12px", color: "#999", fontStyle: "italic", padding: "0 16px" }}>
                  No completions yet…
                </div>
              )}

              {vec.completions?.map(
                (comp: VectorCompletionsResponseStreamingAgentCompletionChunk, ci: number) => {
                  const compError = comp.error
                    ? { code: comp.error.code, message: comp.error.message }
                    : null;
                  return (
                    <AgentCompletionChat
                      key={`${pathLabel}-${comp.index ?? ci}`}
                      label={`${pathLabel} — completion #${comp.index ?? ci}`}
                      chunk={comp}
                      error={compError}
                      id={comp.id}
                    />
                  );
                },
              )}

              {vecError && (
                <div
                  className="ac-error-banner"
                  style={{
                    maxWidth: 800,
                    margin: "0 auto 16px",
                    border: "1px solid #f5c6cb",
                    borderRadius: 8,
                  }}
                >
                  Vector-completion error {vecError.code}: {JSON.stringify(vecError.message)}
                </div>
              )}
            </div>
          );
        })}

      {leaves.length === 0 && !topError && (
        <div
          style={{
            maxWidth: 800,
            margin: "0 auto 24px",
            padding: 16,
            color: "#999",
            fontStyle: "italic",
            textAlign: "center",
          }}
        >
          Waiting for execution…
        </div>
      )}

      {topError && (
        <div
          className="ac-error-banner"
          style={{
            maxWidth: 800,
            margin: "0 auto 24px",
            border: "1px solid #f5c6cb",
            borderRadius: 8,
          }}
        >
          Error {topError.code}: {JSON.stringify(topError.message)}
        </div>
      )}
    </div>
  );
}
