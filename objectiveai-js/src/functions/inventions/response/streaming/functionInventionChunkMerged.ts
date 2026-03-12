import type { FunctionsInventionsResponseStreamingFunctionInventionChunk } from "./functionInventionChunk";
import { functionsInventionsResponseStreamingAgentCompletionChunkMergedList } from "./agentCompletionChunkMerged";
import { agentCompletionsResponseUsageMerged } from "../../../../agent/completions/response/usageMerged";

export function functionsInventionsResponseStreamingFunctionInventionChunkMerged(
  a: FunctionsInventionsResponseStreamingFunctionInventionChunk,
  b: FunctionsInventionsResponseStreamingFunctionInventionChunk,
): [FunctionsInventionsResponseStreamingFunctionInventionChunk, boolean] {
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
