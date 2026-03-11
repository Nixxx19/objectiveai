import { z } from "zod";

export const FunctionsListFunctionProfilePairsSourceSchema = z.literal("objectiveai").describe("Source filter for listing function-profile pairs.").meta({ title: "functions.ListFunctionProfilePairsSource" });
export type FunctionsListFunctionProfilePairsSource = z.infer<typeof FunctionsListFunctionProfilePairsSourceSchema>;
