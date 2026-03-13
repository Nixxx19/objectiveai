import { agentCompletionChunkMerged, agentCompletionChunkNormalized } from "../../../../wasm/loader.js";
import type { AgentCompletionsResponseStreamingAgentCompletionChunk } from "./agentCompletionChunk";

export function wasmAgentCompletionsResponseStreamingAgentCompletionChunkMerged(a: AgentCompletionsResponseStreamingAgentCompletionChunk, b: AgentCompletionsResponseStreamingAgentCompletionChunk): AgentCompletionsResponseStreamingAgentCompletionChunk {
  return JSON.parse(agentCompletionChunkMerged(a, b));
}

export function wasmAgentCompletionsResponseStreamingAgentCompletionChunkNormalized(a: AgentCompletionsResponseStreamingAgentCompletionChunk): AgentCompletionsResponseStreamingAgentCompletionChunk {
  return JSON.parse(agentCompletionChunkNormalized(a));
}
