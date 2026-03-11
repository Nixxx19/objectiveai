import { z } from "zod";
import { AgentWithFallbacksAndCountAgentAgentBaseSchema } from "../agent/withFallbacksAndCount";

export const EnsembleEnsembleBaseSchema = z.object({
  agents: z.array(AgentWithFallbacksAndCountAgentAgentBaseSchema).describe("The LLMs in this ensemble, with optional counts and fallbacks."),
}).describe("The base configuration for an Ensemble (without computed ID).\n\nContains a list of agent configurations that will be validated, deduplicated,\nand sorted when converting to [`Ensemble`].").meta({ title: "ensemble.EnsembleBase" });
export type EnsembleEnsembleBase = z.infer<typeof EnsembleEnsembleBaseSchema>;
