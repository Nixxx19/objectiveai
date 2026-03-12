import { merge } from "../../../../merge";
import type { VectorCompletionsResponseStreamingAgentCompletionChunk } from "./agentCompletionChunk";
import { agentCompletionsResponseStreamingMessageChunkMergedList } from "../../../../agent/completions/response/streaming/messageChunkMerged";
import { agentCompletionsResponseUsageMerged } from "../../../../agent/completions/response/usageMerged";

export function vectorCompletionsResponseStreamingAgentCompletionChunkMerged(
  a: VectorCompletionsResponseStreamingAgentCompletionChunk,
  b: VectorCompletionsResponseStreamingAgentCompletionChunk,
): [VectorCompletionsResponseStreamingAgentCompletionChunk, boolean] {
  let changed = false;

  const [messages, c1] = agentCompletionsResponseStreamingMessageChunkMergedList(a.messages, b.messages);
  if (c1) changed = true;

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
  if (a.error == null && b.error != null) {
    error = b.error;
    changed = true;
  }

  if (!changed) return [a, false];
  return [{
    index: a.index,
    id: a.id,
    created: a.created,
    messages,
    object: a.object,
    ...(usage != null ? { usage } : {}),
    upstream: a.upstream,
    ...(error != null ? { error } : {}),
  }, true];
}

export function vectorCompletionsResponseStreamingAgentCompletionChunkMergedList(
  a: VectorCompletionsResponseStreamingAgentCompletionChunk[],
  b: VectorCompletionsResponseStreamingAgentCompletionChunk[],
): [VectorCompletionsResponseStreamingAgentCompletionChunk[], boolean] {
  let changed = false;
  const result = [...a];
  for (const bItem of b) {
    const existingIdx = result.findIndex((x) => x.index === bItem.index);
    if (existingIdx !== -1) {
      const [merged, c] = vectorCompletionsResponseStreamingAgentCompletionChunkMerged(result[existingIdx], bItem);
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
