import { validateEnsembleLlm } from "src/wasm/objectiveai_wasm_js";
import { EnsembleLlm, EnsembleLlmBase } from "./ensemble_llm";

export function validate(ensemble: EnsembleLlmBase): EnsembleLlm {
  return validateEnsembleLlm(ensemble) as EnsembleLlm;
}
