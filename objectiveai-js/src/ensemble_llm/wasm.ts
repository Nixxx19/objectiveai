import { mapsToRecords } from "src/mapsToRecords";
import { validateEnsembleLlm } from "../wasm/loader.js";
import { EnsembleLlm, EnsembleLlmBase } from "./ensembleLlm";

export function validate(ensemble: EnsembleLlmBase): EnsembleLlm {
  const value = validateEnsembleLlm(ensemble);
  const unmapped = mapsToRecords(value);
  return unmapped as EnsembleLlm;
}
