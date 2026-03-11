import { z } from "zod";

export const FunctionsProfilesUsageProfileSchema = z.object({
  requests: z.number().int().min(0).meta({ format: "uint64" }).describe("Total number of requests made with this profile."),
  completion_tokens: z.number().int().min(0).meta({ format: "uint64" }).describe("Total completion tokens used."),
  prompt_tokens: z.number().int().min(0).meta({ format: "uint64" }).describe("Total prompt tokens used."),
  total_cost: z.union([z.string().regex(new RegExp("^-?\\d+(\\.\\d+)?([eE]\\d+)?$")), z.number()]).describe("Total cost incurred."),
}).describe("Usage statistics for a profile.").meta({ title: "functions.profiles.UsageProfile" });
export type FunctionsProfilesUsageProfile = z.infer<typeof FunctionsProfilesUsageProfileSchema>;
