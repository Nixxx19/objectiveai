import { z } from "zod";

export const EnsembleUsageEnsembleSchema = z.object({
  requests: z.number().int().min(0).meta({ format: "uint64" }).describe("Total number of requests made with this Ensemble."),
  completion_tokens: z.number().int().min(0).meta({ format: "uint64" }).describe("Total completion tokens generated across all agents."),
  prompt_tokens: z.number().int().min(0).meta({ format: "uint64" }).describe("Total prompt tokens processed across all agents."),
  total_cost: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("Total cost incurred."),
}).describe("Usage statistics for an Ensemble.").meta({ title: "ensemble.UsageEnsemble" });
export type EnsembleUsageEnsemble = z.infer<typeof EnsembleUsageEnsembleSchema>;
