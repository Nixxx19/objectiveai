import { z } from "zod";

export const VectorCompletionsRequestProfileEntrySchema = z.object({
  weight: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("The weight for this agent in the ensemble. Must be in [0, 1]."),
  invert: z.boolean().nullable().describe("If true, invert this agent's vote distribution before combining.\n\nWhen omitted or false, the vote distribution is used as-is.").optional(),
}).describe("An entry in a profile with an explicit weight and optional invert flag.").meta({ title: "vector.completions.request.ProfileEntry" });
export type VectorCompletionsRequestProfileEntry = z.infer<typeof VectorCompletionsRequestProfileEntrySchema>;
