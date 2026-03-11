import { z } from "zod";
import { EnsembleListEnsembleItemSchema } from "./listEnsembleItem";

export const EnsembleListEnsembleSchema = z.object({
  data: z.array(EnsembleListEnsembleItemSchema).describe("The list of Ensemble summaries."),
}).describe("Response containing a list of Ensembles.").meta({ title: "ensemble.ListEnsemble" });
export type EnsembleListEnsemble = z.infer<typeof EnsembleListEnsembleSchema>;
