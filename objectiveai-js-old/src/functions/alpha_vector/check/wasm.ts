import {
  alphaCheckLeafVectorFunction as wasmAlphaCheckLeafVectorFunction,
  alphaCheckBranchVectorFunction as wasmAlphaCheckBranchVectorFunction,
} from "../../../wasm/loader.js";
import type { AlphaVectorRemoteFunction } from "../function.js";
import type { RemoteFunction } from "../../function.js";

/**
 * Alpha check for a leaf vector function (depth 0, vector output).
 *
 * Throws a descriptive error string on failure.
 */
export function alphaCheckLeafVectorFunction(
  fn: AlphaVectorRemoteFunction,
): void {
  wasmAlphaCheckLeafVectorFunction(fn);
}

/**
 * Alpha check for a branch vector function (depth > 0, vector output).
 *
 * `children` is an optional map of child function name → RemoteFunction for
 * validating placeholder task inputs against child function input schemas.
 *
 * Throws a descriptive error string on failure.
 */
export function alphaCheckBranchVectorFunction(
  fn: AlphaVectorRemoteFunction,
  children?: Record<string, RemoteFunction>,
): void {
  wasmAlphaCheckBranchVectorFunction(fn, children);
}
