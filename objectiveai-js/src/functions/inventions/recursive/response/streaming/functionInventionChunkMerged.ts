import type { FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunk } from "./functionInventionChunk";
import { functionsInventionsResponseStreamingAgentCompletionChunkMergedList } from "../../../response/streaming/agentCompletionChunkMerged";
import { agentCompletionsResponseUsageMerged } from "../../../../../agent/completions/response/usageMerged";

export function functionsInventionsRecursiveResponseStreamingFunctionInventionChunkMerged(
  a: FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunk,
  b: FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunk,
): [FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunk, boolean] {
  let changed = false;

  const [completions, c1] = functionsInventionsResponseStreamingAgentCompletionChunkMergedList(a.completions, b.completions);
  if (c1) changed = true;

  let state = a.state;
  if (b.state != null) {
    state = b.state;
    changed = true;
  }

  let path = a.path;
  if (b.path != null) {
    path = b.path;
    changed = true;
  }

  let fn = a.function;
  if (b.function != null) {
    fn = b.function;
    changed = true;
  }

  let usage = a.usage;
  if (a.usage != null && b.usage != null) {
    const [merged, c] = agentCompletionsResponseUsageMerged(a.usage, b.usage);
    usage = merged;
    if (c) changed = true;
  } else if (b.usage != null) {
    usage = b.usage;
    changed = true;
  }

  let error = a.error;
  if (b.error != null) {
    error = b.error;
    changed = true;
  }

  if (!changed) return [a, false];
  return [{
    index: a.index,
    id: a.id,
    completions,
    ...(state != null ? { state } : {}),
    ...(path != null ? { path } : {}),
    ...(fn != null ? { function: fn } : {}),
    created: a.created,
    object: a.object,
    ...(usage != null ? { usage } : {}),
    ...(error != null ? { error } : {}),
  }, true];
}

export function functionsInventionsRecursiveResponseStreamingFunctionInventionChunkMergedList(
  a: FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunk[],
  b: FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunk[],
): [FunctionsInventionsRecursiveResponseStreamingFunctionInventionChunk[], boolean] {
  let changed = false;
  const result = [...a];
  for (const bItem of b) {
    const existingIdx = result.findIndex((x) => x.index === bItem.index);
    if (existingIdx !== -1) {
      const [merged, c] = functionsInventionsRecursiveResponseStreamingFunctionInventionChunkMerged(result[existingIdx], bItem);
      if (c) {
        result[existingIdx] = merged;
        changed = true;
      }
    } else {
      result.push(bItem);
      changed = true;
    }
  }
  if (!changed) return [a, false];
  return [result, true];
}
