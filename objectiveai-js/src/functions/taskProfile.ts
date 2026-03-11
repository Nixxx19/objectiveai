import { z } from "zod";
import { FunctionsInlineProfileSchema } from "./inlineProfile";
import { FunctionsRemoteSchema } from "./remote";

export const FunctionsTaskProfileSchema = z.union([z.object({
  remote: FunctionsRemoteSchema.describe("The remote source where the profile is hosted."),
  owner: z.string().describe("Repository owner."),
  repository: z.string().describe("Repository name."),
  commit: z.string().nullable().describe("Git commit SHA. Highly recommended for remote profiles to\nensure compatibility if the referenced profile's shape changes.").optional(),
}).describe("Profile for a nested function task (references another profile)."), z.lazy(() => FunctionsInlineProfileSchema).describe("Inline profile for a task (tasks-based or auto)."), z.record(z.string(), z.unknown()).describe("Placeholder task — no configuration needed, output is fixed.")]).describe("Configuration for a single task within a Profile.\n\nEach variant corresponds to a task type in the Function definition.").meta({ title: "functions.TaskProfile" });
export type FunctionsTaskProfile = z.infer<typeof FunctionsTaskProfileSchema>;
