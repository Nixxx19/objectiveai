import { z } from "zod";
import { FunctionsProfilesListProfilesSourceSchema } from "./listProfilesSource";

export const FunctionsProfilesListProfilesQueryParametersSchema = z.object({
  source: FunctionsProfilesListProfilesSourceSchema.nullable().describe("Optional source filter for listing profiles.").optional(),
}).describe("Query parameters for the list profiles endpoint.").meta({ title: "functions.profiles.ListProfilesQueryParameters" });
export type FunctionsProfilesListProfilesQueryParameters = z.infer<typeof FunctionsProfilesListProfilesQueryParametersSchema>;
