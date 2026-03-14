import { agentCompletionChunkMerged, agentCompletionChunkNormalized, agentCompletionChunkToUnary } from "../../../../wasm/loader.js";
import type { AgentCompletionsResponseStreamingAgentCompletionChunk } from "./agentCompletionChunk";
import type { AgentCompletionsResponseUnaryAgentCompletion } from "../unary/agentCompletion";

export function wasmAgentCompletionsResponseStreamingAgentCompletionChunkMerged(a: AgentCompletionsResponseStreamingAgentCompletionChunk, b: AgentCompletionsResponseStreamingAgentCompletionChunk): AgentCompletionsResponseStreamingAgentCompletionChunk {
  return JSON.parse(agentCompletionChunkMerged(a, b));
}

export function wasmAgentCompletionsResponseStreamingAgentCompletionChunkNormalized(a: AgentCompletionsResponseStreamingAgentCompletionChunk): AgentCompletionsResponseStreamingAgentCompletionChunk {
  return JSON.parse(agentCompletionChunkNormalized(a));
}

export function wasmAgentCompletionsResponseStreamingAgentCompletionChunkToUnary(a: AgentCompletionsResponseStreamingAgentCompletionChunk): AgentCompletionsResponseUnaryAgentCompletion {
  return JSON.parse(agentCompletionChunkToUnary(a));
}
