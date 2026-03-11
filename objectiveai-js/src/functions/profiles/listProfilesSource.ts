import { z } from "zod";

export const FunctionsProfilesListProfilesSourceSchema = z.enum(["all","mock","filesystem","objectiveai"]).describe("Source filter for listing profiles.").meta({ title: "functions.profiles.ListProfilesSource" });
export type FunctionsProfilesListProfilesSource = z.infer<typeof FunctionsProfilesListProfilesSourceSchema>;
