import { z } from "zod";
import { FunctionsTaskProfileSchema } from "./taskProfile";
import { VectorCompletionsRequestProfileSchema } from "../vector/completions/request/profile";

export const FunctionsRemoteTasksProfileSchema = z.object({
  description: z.string().describe("Human-readable description of the profile."),
  tasks: z.array(FunctionsTaskProfileSchema).describe("Configuration for each task in the corresponding Function."),
  profile: VectorCompletionsRequestProfileSchema.describe("Weights for each Task in the corresponding Function.\n\nMust have the same length as `tasks`. Can be either:\n- A vector of decimals (legacy representation), or\n- A vector of objects with `weight` and optional `invert` fields."),
}).describe("A remote tasks-based profile with full metadata.\n\nStored as `profile.json` in repositories and referenced by\n`remote/owner/repository`.").meta({ title: "functions.RemoteTasksProfile" });
export type FunctionsRemoteTasksProfile = z.infer<typeof FunctionsRemoteTasksProfileSchema>;
