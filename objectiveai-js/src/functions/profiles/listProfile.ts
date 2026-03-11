import { z } from "zod";
import { FunctionsProfilesListProfileItemSchema } from "./listProfileItem";

export const FunctionsProfilesListProfileSchema = z.object({
  data: z.array(FunctionsProfilesListProfileItemSchema).describe("List of available profiles."),
}).describe("Response from listing profiles.").meta({ title: "functions.profiles.ListProfile" });
export type FunctionsProfilesListProfile = z.infer<typeof FunctionsProfilesListProfileSchema>;
