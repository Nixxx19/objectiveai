import { merge } from "../../../../merge";
import type { AgentCompletionsResponseStreamingAgentCompletionChunk } from "./agentCompletionChunk";
import { agentCompletionsResponseStreamingMessageChunkMergedList } from "./messageChunkMerged";
import { agentCompletionsResponseUsageMerged } from "../usageMerged";

export function agentCompletionsResponseStreamingAgentCompletionChunkMerged(
  a: AgentCompletionsResponseStreamingAgentCompletionChunk,
  b: AgentCompletionsResponseStreamingAgentCompletionChunk,
): [AgentCompletionsResponseStreamingAgentCompletionChunk, boolean] {
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
