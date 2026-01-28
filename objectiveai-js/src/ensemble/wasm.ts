import { mapsToRecords } from "src/mapsToRecords.js";
import { validateEnsemble } from "../wasm/loader.js";
import { Ensemble, EnsembleBase } from "./ensemble";

export function validate(ensemble: EnsembleBase): Ensemble {
  const value = validateEnsemble(ensemble);
  const unmapped = mapsToRecords(value);
  return unmapped as Ensemble;
}
