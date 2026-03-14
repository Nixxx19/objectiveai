import { functionInventionChunkMerged, functionInventionChunkNormalized, functionInventionChunkToUnary } from "../../../../wasm/loader.js";
import type { FunctionsInventionsResponseStreamingFunctionInventionChunk } from "./functionInventionChunk";
import type { FunctionsInventionsResponseUnaryFunctionInvention } from "../unary/functionInvention";

export function wasmFunctionsInventionsResponseStreamingFunctionInventionChunkMerged(a: FunctionsInventionsResponseStreamingFunctionInventionChunk, b: FunctionsInventionsResponseStreamingFunctionInventionChunk): FunctionsInventionsResponseStreamingFunctionInventionChunk {
  return JSON.parse(functionInventionChunkMerged(a, b));
}

export function wasmFunctionsInventionsResponseStreamingFunctionInventionChunkNormalized(a: FunctionsInventionsResponseStreamingFunctionInventionChunk): FunctionsInventionsResponseStreamingFunctionInventionChunk {
  return JSON.parse(functionInventionChunkNormalized(a));
}

export function wasmFunctionsInventionsResponseStreamingFunctionInventionChunkToUnary(a: FunctionsInventionsResponseStreamingFunctionInventionChunk): FunctionsInventionsResponseUnaryFunctionInvention {
  return JSON.parse(functionInventionChunkToUnary(a));
}
