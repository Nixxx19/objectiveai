import { z } from "zod";
import { FunctionsGetFunctionSchema } from "./getFunction";
import { FunctionsProfilesGetProfileSchema } from "./profiles/getProfile";

export const FunctionsGetFunctionProfilePairSchema = z.object({
  function: FunctionsGetFunctionSchema.describe("The function."),
  profile: FunctionsProfilesGetProfileSchema.describe("The profile."),
}).describe("Response from getting a function-profile pair.").meta({ title: "functions.GetFunctionProfilePair" });
export type FunctionsGetFunctionProfilePair = z.infer<typeof FunctionsGetFunctionProfilePairSchema>;
