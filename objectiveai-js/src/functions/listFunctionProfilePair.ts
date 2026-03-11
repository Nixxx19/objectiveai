import { z } from "zod";
import { FunctionsListFunctionProfilePairItemSchema } from "./listFunctionProfilePairItem";

export const FunctionsListFunctionProfilePairSchema = z.object({
  data: z.array(FunctionsListFunctionProfilePairItemSchema).describe("List of available function-profile pairs."),
}).describe("Response from listing function-profile pairs.").meta({ title: "functions.ListFunctionProfilePair" });
export type FunctionsListFunctionProfilePair = z.infer<typeof FunctionsListFunctionProfilePairSchema>;
