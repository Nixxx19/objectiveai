import {
  checkScalarFields as wasmCheckScalarFields,
  checkVectorFields as wasmCheckVectorFields,
} from "../../wasm/loader.js";
import { ScalarFieldsValidation } from "./scalarFields.js";
import { VectorFieldsValidation } from "./vectorFields.js";

/**
 * Validates that a scalar function's input_schema produces enough diverse
 * example inputs.
 *
 * Throws a descriptive error string on failure.
 */
export function checkScalarFields(fields: ScalarFieldsValidation): void {
  wasmCheckScalarFields(fields);
}

/**
 * Validates that a vector function's output_length, input_split, and
 * input_merge expressions work correctly together.
 *
 * Generates diverse, randomized example inputs from the input_schema and
 * performs round-trip testing: split → merge → compare. Throws a descriptive
 * error string on failure.
 */
export function checkVectorFields(fields: VectorFieldsValidation): void {
  wasmCheckVectorFields(fields);
}
