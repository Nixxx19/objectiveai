import { z } from "zod";
import { FunctionsRemoteSchema } from "../remote";

export const FunctionsProfilesListProfileItemSchema = z.object({
  remote: FunctionsRemoteSchema.describe("The remote source where the profile is hosted."),
  owner: z.string().describe("Repository owner."),
  repository: z.string().describe("Repository name."),
  commit: z.string().describe("Git commit SHA."),
}).describe("A profile in a list response.").meta({ title: "functions.profiles.ListProfileItem" });
export type FunctionsProfilesListProfileItem = z.infer<typeof FunctionsProfilesListProfileItemSchema>;
