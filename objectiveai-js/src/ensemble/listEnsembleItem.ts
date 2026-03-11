import { z } from "zod";

export const EnsembleListEnsembleItemSchema = z.object({
  id: z.string().describe("The unique content-addressed ID of the Ensemble."),
}).describe("Summary information for a listed Ensemble.").meta({ title: "ensemble.ListEnsembleItem" });
export type EnsembleListEnsembleItem = z.infer<typeof EnsembleListEnsembleItemSchema>;
