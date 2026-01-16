import { validateEnsemble } from "src/wasm/objectiveai_wasm_js";
import { Ensemble, EnsembleBase } from "./ensemble";

export function validate(ensemble: EnsembleBase): Ensemble {
  return validateEnsemble(ensemble) as Ensemble;
}
