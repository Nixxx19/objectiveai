import {
  alphaCheckLeafScalarFunction as wasmAlphaCheckLeafScalarFunction,
  alphaCheckBranchScalarFunction as wasmAlphaCheckBranchScalarFunction,
} from "../../../wasm/loader.js";
import type { AlphaScalarRemoteFunction } from "../function.js";
import type { RemoteFunction } from "../../function.js";

/**
 * Alpha check for a leaf scalar function (depth 0, scalar output).
 *
 * Throws a descriptive error string on failure.
 */
export function alphaCheckLeafScalarFunction(
  fn: AlphaScalarRemoteFunction,
): void {
  wasmAlphaCheckLeafScalarFunction(fn);
}

/**
 * Alpha check for a branch scalar function (depth > 0, scalar output).
 *
 * `children` is an optional map of child function name → RemoteFunction for
 * validating placeholder task inputs against child function input schemas.
 *
 * Throws a descriptive error string on failure.
 */
export function alphaCheckBranchScalarFunction(
  fn: AlphaScalarRemoteFunction,
  children?: Record<string, RemoteFunction>,
): void {
  wasmAlphaCheckBranchScalarFunction(fn, children);
}
