import { functionInventionChunkMerged, functionInventionChunkNormalized } from "../../../../wasm/loader.js";
import type { FunctionsInventionsResponseStreamingFunctionInventionChunk } from "./functionInventionChunk";

export function wasmFunctionsInventionsResponseStreamingFunctionInventionChunkMerged(a: FunctionsInventionsResponseStreamingFunctionInventionChunk, b: FunctionsInventionsResponseStreamingFunctionInventionChunk): FunctionsInventionsResponseStreamingFunctionInventionChunk {
  return JSON.parse(functionInventionChunkMerged(a, b));
}

export function wasmFunctionsInventionsResponseStreamingFunctionInventionChunkNormalized(a: FunctionsInventionsResponseStreamingFunctionInventionChunk): FunctionsInventionsResponseStreamingFunctionInventionChunk {
  return JSON.parse(functionInventionChunkNormalized(a));
}
