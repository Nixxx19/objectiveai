import { validateEnsemble } from "../wasm/loader.js";
import type { EnsembleEnsembleBase } from "./ensembleBase";
import type { EnsembleEnsemble } from "./ensemble";

export function wasmEnsembleValidateEnsemble(ensemble: EnsembleEnsembleBase): EnsembleEnsemble {
  return JSON.parse(validateEnsemble(ensemble));
}
