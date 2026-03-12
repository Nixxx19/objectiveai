import { ObjectiveAI, type RequestOptions } from "../client";
import type { EnsembleListEnsemble } from "./listEnsemble";
import type { EnsembleGetEnsemble } from "./getEnsemble";
import type { EnsembleUsageEnsemble } from "./usageEnsemble";

export function ensembleListEnsembles(
  client: ObjectiveAI,
  options?: RequestOptions,
): Promise<EnsembleListEnsemble> {
  return client.get_unary<EnsembleListEnsemble>("/ensembles", undefined, options);
}

export function ensembleGetEnsemble(
  client: ObjectiveAI,
  ensembleId: string,
  options?: RequestOptions,
): Promise<EnsembleGetEnsemble> {
  return client.get_unary<EnsembleGetEnsemble>(
    `/ensembles/${ensembleId}`,
    undefined,
    options,
  );
}

export function ensembleGetEnsembleUsage(
  client: ObjectiveAI,
  ensembleId: string,
  options?: RequestOptions,
): Promise<EnsembleUsageEnsemble> {
  return client.get_unary<EnsembleUsageEnsemble>(
    `/ensembles/${ensembleId}/usage`,
    undefined,
    options,
  );
}
