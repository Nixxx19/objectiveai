import { vectorCompletionChunkMerged, vectorCompletionChunkNormalized } from "../../../../wasm/loader.js";
import type { VectorCompletionsResponseStreamingVectorCompletionChunk } from "./vectorCompletionChunk";

export function wasmVectorCompletionsResponseStreamingVectorCompletionChunkMerged(a: VectorCompletionsResponseStreamingVectorCompletionChunk, b: VectorCompletionsResponseStreamingVectorCompletionChunk): VectorCompletionsResponseStreamingVectorCompletionChunk {
  return JSON.parse(vectorCompletionChunkMerged(a, b));
}

export function wasmVectorCompletionsResponseStreamingVectorCompletionChunkNormalized(a: VectorCompletionsResponseStreamingVectorCompletionChunk): VectorCompletionsResponseStreamingVectorCompletionChunk {
  return JSON.parse(vectorCompletionChunkNormalized(a));
}
