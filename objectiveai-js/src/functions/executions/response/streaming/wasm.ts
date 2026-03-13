import { functionExecutionChunkMerged, functionExecutionChunkNormalized } from "../../../../wasm/loader.js";
import type { FunctionsExecutionsResponseStreamingFunctionExecutionChunk } from "./functionExecutionChunk";

export function wasmFunctionsExecutionsResponseStreamingFunctionExecutionChunkMerged(a: FunctionsExecutionsResponseStreamingFunctionExecutionChunk, b: FunctionsExecutionsResponseStreamingFunctionExecutionChunk): FunctionsExecutionsResponseStreamingFunctionExecutionChunk {
  return JSON.parse(functionExecutionChunkMerged(a, b));
}

export function wasmFunctionsExecutionsResponseStreamingFunctionExecutionChunkNormalized(a: FunctionsExecutionsResponseStreamingFunctionExecutionChunk): FunctionsExecutionsResponseStreamingFunctionExecutionChunk {
  return JSON.parse(functionExecutionChunkNormalized(a));
}
