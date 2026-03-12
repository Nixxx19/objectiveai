import type { FunctionsExecutionsResponseStreamingReasoningSummaryChunk } from "./reasoningSummaryChunk";
import { agentCompletionsResponseStreamingMessageChunkMergedList } from "../../../../agent/completions/response/streaming/messageChunkMerged";
import { agentCompletionsResponseUsageMerged } from "../../../../agent/completions/response/usageMerged";

export function functionsExecutionsResponseStreamingReasoningSummaryChunkMerged(
  a: FunctionsExecutionsResponseStreamingReasoningSummaryChunk,
  b: FunctionsExecutionsResponseStreamingReasoningSummaryChunk,
): [FunctionsExecutionsResponseStreamingReasoningSummaryChunk, boolean] {
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
    id: a.id,
    created: a.created,
    messages,
    object: a.object,
    ...(usage != null ? { usage } : {}),
    upstream: a.upstream,
    ...(error != null ? { error } : {}),
  }, true];
}
